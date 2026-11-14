//! Native adaptation of ScriptingContext::performRegistration. Protocol
//! generators stay with their ScriptAsset; only utility values enter require.
use super::*;
use std::collections::HashMap;

const DEPENDENCIES: &str = "_RIVE_REGISTRATION_DEPENDENCIES";

pub(super) struct RegisteredProtocolProgram {
    pub vm: Rc<()>,
    pub program: ScriptProgram,
}

struct RegistrationScope {
    lua: Lua,
    previous: Value,
}
impl Drop for RegistrationScope {
    fn drop(&mut self) {
        // Restore an enclosing registration scope during nested host calls.
        let _ = self
            .lua
            .set_named_registry_value(DEPENDENCIES, self.previous.clone());
    }
}

pub(super) fn register(
    vm: &ScriptVm,
    scripts: &[ScriptAssetRegistration<'_>],
) -> Vec<ScriptAssetRegistrationResult> {
    let mut results: Vec<_> = scripts
        .iter()
        .map(|_| ScriptAssetRegistrationResult::default())
        .collect();
    let mut run = || -> Result<()> {
        vm.install_rive_globals()?;
        let dependencies = vm.lua.create_table();
        let lookup: BTreeMap<_, _> = scripts
            .iter()
            .enumerate()
            .map(|(index, script)| (script.name, index))
            .collect();
        for script in scripts {
            let values = vm.lua.create_table();
            for dependency in &script.missing_dependencies {
                values.raw_set(dependency.as_str(), true)?;
            }
            dependencies.raw_set(script.name, values)?;
        }
        let _scope = RegistrationScope {
            lua: vm.lua.clone(),
            previous: vm
                .lua
                .named_registry_value(DEPENDENCIES)
                .unwrap_or(Value::Nil),
        };
        vm.lua
            .set_named_registry_value(DEPENDENCIES, dependencies.clone())?;
        let mut pending = BTreeSet::new();
        for (index, (script, result)) in scripts.iter().zip(results.iter_mut()).enumerate() {
            if !matches!(vm.registered_module(script.name)?, Value::Nil) {
                continue;
            }
            attempt(vm, scripts, index, result, &dependencies, &mut pending)?;
        }
        // Exactly one dependency-ordered retry pass. Ordinary script failures
        // do not acquire retries merely because another module made progress.
        let roots: Vec<_> = pending.iter().copied().collect();
        let order = dependency_order(&roots, |index| {
            let Some(script) = scripts.get(index) else {
                return Ok(Vec::new());
            };
            let deps: Table = dependencies.raw_get(script.name)?;
            deps.pairs::<String, bool>()
                .filter_map(|entry| match entry {
                    Ok((name, _)) => lookup.get(name.as_str()).copied().map(Ok),
                    Err(error) => Some(Err(error)),
                })
                .collect()
        })?;
        for index in order {
            let (Some(script), Some(result)) = (scripts.get(index), results.get_mut(index)) else {
                continue;
            };
            if !matches!(vm.registered_module(script.name)?, Value::Nil) {
                continue;
            }
            let deps: Table = dependencies.raw_get(script.name)?;
            if deps.pairs::<String, bool>().next().is_some() {
                continue;
            }
            attempt(vm, scripts, index, result, &dependencies, &mut pending)?;
        }
        for (script, result) in scripts.iter().zip(results.iter_mut()) {
            let deps: Table = dependencies.raw_get(script.name)?;
            result.missing_dependencies = deps
                .pairs::<String, bool>()
                .map(|value| value.map(|(name, _)| name))
                .collect::<Result<_>>()?;
        }
        Ok(())
    };
    if let Err(error) = run() {
        let error = vm.script_error(error);
        for result in &mut results {
            if !result.completed {
                result.error = Some(error.clone());
            }
        }
    }
    results
}

const CYCLE: &str = "cyclic script module dependency";

/// The order the retry pass visits pending scripts in: `roots` and the
/// scripts they require, each after what it requires; an error when those
/// requires contain a cycle.
///
/// Iterative, with an explicit stack: each script's dependencies are read
/// once per walk, so the work is linear in scripts plus dependencies.
///
/// For every input the earlier recursive `sort` accepted, the order is the
/// one it produced. Its rule: roots are taken from the end of `roots`
/// (descending script index); a script is placed after a depth-first walk
/// of its dependencies in their table order; and each time a script is
/// placed, the next root is started on top of the walk in progress (the
/// recursive version called itself after every placement). That
/// interleaving reported a cycle whenever a later root reached a script
/// still open in an earlier root's walk, although there was none (E1
/// inventory 4.2). On such a conflict the roots are walked again one after
/// the other, which stops only on a real cycle.
fn dependency_order(
    roots: &[usize],
    mut dependencies_of: impl FnMut(usize) -> Result<Vec<usize>>,
) -> Result<Vec<usize>> {
    let stop = match walk(roots, Interleave::OnPlacement, &mut dependencies_of) {
        Ok(order) => return Ok(order),
        Err(Stop::EarlierRoot) => match walk(roots, Interleave::Never, &mut dependencies_of) {
            Ok(order) => return Ok(order),
            Err(stop) => stop,
        },
        Err(stop) => stop,
    };
    Err(match stop {
        Stop::Lua(error) => error,
        Stop::Cycle | Stop::EarlierRoot => Error::runtime(CYCLE),
    })
}

/// Why a walk stopped before placing every script.
enum Stop {
    /// A script requires one that is open in the same root's walk: a cycle.
    Cycle,
    /// The next root, or a dependency, is open in an earlier root's walk.
    EarlierRoot,
    /// Reading a dependency table failed.
    Lua(Error),
}

/// When the next root starts.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Interleave {
    /// After every placement, on top of the walk in progress (the recursive
    /// version's order).
    OnPlacement,
    /// When the previous root's walk is finished.
    Never,
}

#[derive(Clone, Copy)]
enum Mark {
    /// On the stack, in the walk started from this root.
    Open { root: usize },
    Placed,
}

/// A script on the stack and the dependencies it has not visited yet.
struct Frame {
    script: usize,
    root: usize,
    dependencies: std::vec::IntoIter<usize>,
}

fn walk(
    roots: &[usize],
    interleave: Interleave,
    dependencies_of: &mut impl FnMut(usize) -> Result<Vec<usize>>,
) -> std::result::Result<Vec<usize>, Stop> {
    let mut roots = roots.iter().rev().copied();
    let mut marks = HashMap::new();
    let mut stack: Vec<Frame> = Vec::new();
    let mut order = Vec::new();
    let mut open = |script: usize,
                    root: usize,
                    marks: &mut HashMap<usize, Mark>,
                    stack: &mut Vec<Frame>|
     -> std::result::Result<(), Stop> {
        marks.insert(script, Mark::Open { root });
        let dependencies = dependencies_of(script).map_err(Stop::Lua)?;
        stack.push(Frame {
            script,
            root,
            dependencies: dependencies.into_iter(),
        });
        Ok(())
    };
    loop {
        let Some(frame) = stack.last_mut() else {
            // Seed the next walk from the pending roots.
            let Some(root) = roots.next() else {
                return Ok(order);
            };
            if !marks.contains_key(&root) {
                open(root, root, &mut marks, &mut stack)?;
            }
            continue;
        };
        if let Some(dependency) = frame.dependencies.next() {
            let root = frame.root;
            match marks.get(&dependency) {
                None => open(dependency, root, &mut marks, &mut stack)?,
                Some(Mark::Placed) => {}
                Some(Mark::Open { root: open_in }) if *open_in == root => {
                    return Err(Stop::Cycle);
                }
                Some(Mark::Open { .. }) => return Err(Stop::EarlierRoot),
            }
            continue;
        }
        // Everything this script requires is placed: place it.
        let script = frame.script;
        stack.pop();
        marks.insert(script, Mark::Placed);
        order.push(script);
        if interleave == Interleave::OnPlacement
            && let Some(root) = roots.next()
        {
            match marks.get(&root) {
                None => open(root, root, &mut marks, &mut stack)?,
                Some(Mark::Placed) => {}
                Some(Mark::Open { .. }) => return Err(Stop::EarlierRoot),
            }
        }
    }
}

fn attempt(
    vm: &ScriptVm,
    scripts: &[ScriptAssetRegistration<'_>],
    index: usize,
    result: &mut ScriptAssetRegistrationResult,
    dependencies: &Table,
    pending: &mut BTreeSet<usize>,
) -> Result<()> {
    let Some(script) = scripts.get(index) else {
        return Ok(());
    };
    let registered = (|| -> Result<Option<RuntimeScriptProgram>> {
        let cached = vm.registered_module(script.name)?;
        let value = if matches!(cached, Value::Nil) {
            // ScriptAsset has already stripped/verified the signed envelope.
            let chunk = vm.load_module(script.name, script.bytecode)?;
            vm.reset_execution_budget();
            let value: Value = vm.execute_module(script.name, chunk)?;
            if !script.is_protocol {
                vm.cache_registered_module(script.name, value.clone())?;
            }
            value
        } else {
            cached
        };
        Ok(if script.is_protocol {
            match value {
                Value::Function(generator) => Some(RuntimeScriptProgram::from_backend(
                    RegisteredProtocolProgram {
                        vm: vm.runtime_identity.clone(),
                        program: ScriptProgram { generator },
                    },
                )),
                _ => None, // Successful table return registers no generator ref.
            }
        } else {
            None
        })
    })();
    for (candidate, script) in scripts.iter().enumerate() {
        let deps: Table = dependencies.raw_get(script.name)?;
        if deps.pairs::<String, bool>().next().is_some() {
            pending.insert(candidate);
        }
    }
    match registered {
        Ok(program) => {
            result.completed = true;
            result.program = program;
            result.error = None;
            for candidate in scripts {
                let deps: Table = dependencies.raw_get(candidate.name)?;
                deps.raw_set(script.name, Value::Nil)?;
            }
            pending.remove(&index);
        }
        Err(error) => result.error = Some(vm.script_error(error)),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The recursive `sort` this module had before lane E2a, on a plain
    /// graph: the oracle for the order of every input it accepted.
    fn recursive_sort(roots: &[usize], graph: &[Vec<usize>]) -> std::result::Result<Vec<usize>, ()> {
        fn sort(
            index: usize,
            graph: &[Vec<usize>],
            remaining: &mut Vec<usize>,
            sorted: &mut Vec<usize>,
            visited: &mut BTreeSet<usize>,
            visiting: &mut BTreeSet<usize>,
        ) -> std::result::Result<(), ()> {
            if visited.contains(&index) {
                return Ok(());
            }
            if !visiting.insert(index) {
                return Err(());
            }
            for &dependency in &graph[index] {
                sort(dependency, graph, remaining, sorted, visited, visiting)?;
            }
            if !sorted.contains(&index) {
                sorted.push(index);
            }
            visiting.remove(&index);
            visited.insert(index);
            if let Some(next) = remaining.pop() {
                sort(next, graph, remaining, sorted, visited, visiting)?;
            }
            Ok(())
        }
        let mut remaining = roots.to_vec();
        let mut sorted = Vec::new();
        let (mut visited, mut visiting) = (BTreeSet::new(), BTreeSet::new());
        if let Some(next) = remaining.pop() {
            sort(next, graph, &mut remaining, &mut sorted, &mut visited, &mut visiting)?;
        }
        Ok(sorted)
    }

    /// Whether a cycle is reachable from `roots` (colouring depth-first
    /// search, recursive; graphs here are small).
    fn has_reachable_cycle(roots: &[usize], graph: &[Vec<usize>]) -> bool {
        fn visit(node: usize, graph: &[Vec<usize>], colour: &mut [u8]) -> bool {
            match colour[node] {
                1 => return true,
                2 => return false,
                _ => {}
            }
            colour[node] = 1;
            if graph[node].iter().any(|&next| visit(next, graph, colour)) {
                return true;
            }
            colour[node] = 2;
            false
        }
        let mut colour = vec![0; graph.len()];
        roots.iter().any(|&root| visit(root, graph, &mut colour))
    }

    fn reachable(roots: &[usize], graph: &[Vec<usize>]) -> BTreeSet<usize> {
        let mut seen = BTreeSet::new();
        let mut stack = roots.to_vec();
        while let Some(node) = stack.pop() {
            if seen.insert(node) {
                stack.extend(&graph[node]);
            }
        }
        seen
    }

    fn order(roots: &[usize], graph: &[Vec<usize>]) -> Result<Vec<usize>> {
        dependency_order(roots, |index| Ok(graph[index].clone()))
    }

    fn assert_topological(order: &[usize], roots: &[usize], graph: &[Vec<usize>]) {
        let placed: BTreeMap<usize, usize> =
            order.iter().enumerate().map(|(at, &node)| (node, at)).collect();
        assert_eq!(placed.len(), order.len(), "placed once: {order:?}");
        assert_eq!(
            placed.keys().copied().collect::<BTreeSet<_>>(),
            reachable(roots, graph),
            "{order:?}"
        );
        for (&node, &at) in &placed {
            for dependency in &graph[node] {
                assert!(placed[dependency] < at, "{dependency} before {node}: {order:?}");
            }
        }
    }

    struct XorShift(u64);
    impl XorShift {
        fn next(&mut self, bound: usize) -> usize {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 % bound as u64) as usize
        }
    }

    /// A random graph on `nodes` scripts; acyclic when every edge goes from
    /// a script to one of a lower rank in a random ranking.
    fn random_graph(rng: &mut XorShift, nodes: usize, acyclic: bool) -> (Vec<usize>, Vec<Vec<usize>>) {
        let mut rank: Vec<usize> = (0..nodes).collect();
        for at in (1..nodes).rev() {
            rank.swap(at, rng.next(at + 1));
        }
        let mut graph = vec![Vec::new(); nodes];
        for (node, edges) in graph.iter_mut().enumerate() {
            for _ in 0..rng.next(4) {
                let target = rng.next(nodes);
                if target != node
                    && (!acyclic || rank[target] < rank[node])
                    && !edges.contains(&target)
                {
                    edges.push(target);
                }
            }
        }
        let roots = (0..nodes).filter(|_| rng.next(3) != 0).collect();
        (roots, graph)
    }

    /// E1 4.2's minimal input as a graph: scripts [Y, X, P] pending after
    /// the first pass (C registered), X requires Y, P requires X.
    #[test]
    fn the_minimal_false_cycle_orders_dependencies_first() {
        let graph = vec![vec![], vec![0], vec![1]];
        assert!(recursive_sort(&[0, 1, 2], &graph).is_err(), "the old sort's false cycle");
        assert_eq!(order(&[0, 1, 2], &graph).unwrap(), vec![0, 1, 2]);
    }

    #[test]
    fn a_real_cycle_is_still_reported_with_the_same_text() {
        let graph = vec![vec![1], vec![0]];
        let error = order(&[0, 1], &graph).unwrap_err();
        assert_eq!(error.to_string(), "runtime error: cyclic script module dependency");
        let self_loop = vec![vec![0]];
        assert!(order(&[0], &self_loop).is_err());
    }

    /// Seeded property test: on random graphs the order matches the old
    /// recursive sort wherever that sort succeeded, is a topological order
    /// of exactly the scripts reachable from the roots, and a cycle is
    /// reported exactly when one is reachable.
    #[test]
    fn matches_the_old_order_and_a_reference_sort_on_random_graphs() {
        let mut rng = XorShift(0x5eed_e2a0_0000_0001);
        let (mut acyclic_runs, mut false_cycles_fixed, mut cycles) = (0, 0, 0);
        for round in 0..20_000 {
            let nodes = 1 + rng.next(12);
            let acyclic = round % 4 != 0;
            let (roots, graph) = random_graph(&mut rng, nodes, acyclic);
            let cyclic = has_reachable_cycle(&roots, &graph);
            let ours = order(&roots, &graph);
            match (&ours, recursive_sort(&roots, &graph)) {
                (Ok(order), Ok(old)) => assert_eq!(order, &old, "{roots:?} {graph:?}"),
                (Ok(_), Err(())) => false_cycles_fixed += 1,
                (Err(_), _) => {}
            }
            match ours {
                Ok(order) => {
                    assert!(!cyclic, "missed a cycle: {roots:?} {graph:?}");
                    assert_topological(&order, &roots, &graph);
                    acyclic_runs += 1;
                }
                Err(error) => {
                    assert!(cyclic, "false cycle: {roots:?} {graph:?}");
                    assert_eq!(error.to_string(), "runtime error: cyclic script module dependency");
                    cycles += 1;
                }
            }
        }
        assert!(false_cycles_fixed > 0 && cycles > 0 && acyclic_runs > 10_000);
        eprintln!(
            "dependency_order: {acyclic_runs} ordered, {cycles} real cycles, \
             {false_cycles_fixed} false cycles of the old sort now ordered"
        );
    }

    /// Linear: a chain of 200,000 scripts and a wide fan-out need no
    /// recursion and finish at once.
    #[test]
    fn a_long_chain_needs_no_recursion() {
        let count = 200_000;
        let chain: Vec<Vec<usize>> = (0..count)
            .map(|node| if node == 0 { vec![] } else { vec![node - 1] })
            .collect();
        let roots: Vec<usize> = (0..count).collect();
        let order = order(&roots, &chain).unwrap();
        assert_eq!(order, roots);
    }
}
