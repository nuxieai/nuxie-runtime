//! Complete red entry points for Wave A cases whose prior Rust evidence was only nearby coverage.
//!
//! Each ignored test retains the entire pinned TEST_CASE body, including its fixture,
//! action order, and every assertion. The helper deliberately fails after verifying
//! the retained body shape: removing `#[ignore]` is not a promotion until the body has
//! been transliterated to executable Rust against the runtime under test.

fn pending_literal_port(pinned_cpp_case: &str) {
    assert!(pinned_cpp_case.starts_with("TEST_CASE("));
    assert!(
        pinned_cpp_case.contains("REQUIRE(") || pinned_cpp_case.contains("CHECK("),
        "retained case must include its pinned assertions"
    );
    panic!("expected-red: retained pinned case still needs executable Rust transliteration");
}

#[test]
#[ignore = "expected-red: prior Rust evidence covered only a narrower observable, not this complete pinned case"]
fn component_list_case_01_direct_port_expected_red() {
    pending_literal_port(
        r####"TEST_CASE("Component List Artboard Count", "[component_list]")
{
    auto file = ReadRiveFile("assets/component_list_1.riv");

    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);

    REQUIRE(artboard->find<rive::ArtboardComponentList>("List") != nullptr);
    auto list = artboard->find<rive::ArtboardComponentList>("List");

    artboard->advance(0.0f);

    REQUIRE(list->syncStyleChanges() == true);
    REQUIRE(list->artboardCount() == 8);
    REQUIRE(list->layoutNode(9) == nullptr);
    REQUIRE(list->artboardInstance(9) == nullptr);
    REQUIRE(list->stateMachineInstance(9) == nullptr);
}"####,
    );
}

#[test]
#[ignore = "expected-red: prior Rust evidence covered only a narrower observable, not this complete pinned case"]
fn component_list_case_02_direct_port_expected_red() {
    pending_literal_port(
        r####"TEST_CASE("Component List Artboards & State Machines", "[component_list]")
{
    auto file = ReadRiveFile("assets/component_list_1.riv");

    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);

    REQUIRE(artboard->find<rive::ArtboardComponentList>("List") != nullptr);
    auto list = artboard->find<rive::ArtboardComponentList>("List");

    artboard->advance(0.0f);

    for (int i = 0; i < list->artboardCount(); i++)
    {
        auto artboard = list->artboardInstance(i);
        REQUIRE(artboard != nullptr);
        REQUIRE(artboard->name() == "Item");
        auto sm = list->stateMachineInstance(i);
        REQUIRE(sm != nullptr);
        REQUIRE(sm->artboard() == artboard);
    }
}"####,
    );
}

#[test]
#[ignore = "expected-red: prior Rust evidence covered only a narrower observable, not this complete pinned case"]
fn component_list_case_03_direct_port_expected_red() {
    pending_literal_port(
        r####"TEST_CASE("Component List Artboards Layout Nodes", "[component_list]")
{
    auto file = ReadRiveFile("assets/component_list_1.riv");

    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);

    REQUIRE(artboard->find<rive::ArtboardComponentList>("List") != nullptr);
    auto list = artboard->find<rive::ArtboardComponentList>("List");

    artboard->advance(0.0f);

    for (int i = 0; i < list->artboardCount(); i++)
    {
        auto node = list->layoutNode(i);
        REQUIRE(node != nullptr);
    }
}"####,
    );
}

#[test]
#[ignore = "expected-red: prior Rust evidence covered only a narrower observable, not this complete pinned case"]
fn component_list_case_04_direct_port_expected_red() {
    pending_literal_port(
        r####"TEST_CASE("Component List Artboards Layout Bounds", "[component_list]")
{
    auto file = ReadRiveFile("assets/component_list_1.riv");

    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);

    REQUIRE(artboard->find<rive::ArtboardComponentList>("List") != nullptr);
    auto list = artboard->find<rive::ArtboardComponentList>("List");

    artboard->advance(0.0f);

    for (int i = 0; i < list->artboardCount(); i++)
    {
        auto artboard = list->artboardInstance(i);
        REQUIRE(artboard != nullptr);
        auto bounds = artboard->layoutBounds();
        // Artboards are using Column layout
        REQUIRE(bounds.top() == i * 60);
    }
}"####,
    );
}

#[test]
#[ignore = "expected-red: prior Rust evidence covered only a narrower observable, not this complete pinned case"]
fn component_list_case_05_direct_port_expected_red() {
    pending_literal_port(
        r####"TEST_CASE("Component List Artboards Data Context", "[component_list]")
{
    auto file = ReadRiveFile("assets/component_list_1.riv");

    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);

    REQUIRE(artboard->find<rive::ArtboardComponentList>("List") != nullptr);
    auto list = artboard->find<rive::ArtboardComponentList>("List");

    artboard->advance(0.0f);

    std::vector<std::string> labels =
        {"ONE", "TWO", "THREE", "THREE", "THREE", "THREE", "TWO", "ONE"};
    for (int i = 0; i < list->artboardCount(); i++)
    {
        auto artboard = list->artboardInstance(i);
        REQUIRE(artboard != nullptr);
        auto context = artboard->dataContext();
        auto vmString =
            context->mainViewModelInstance()->propertyValue("Label");
        REQUIRE(vmString->is<rive::ViewModelInstanceString>());
        REQUIRE(
            vmString->as<rive::ViewModelInstanceString>()->propertyValue() ==
            labels[i]);
    }
}"####,
    );
}

#[test]
#[ignore = "expected-red: prior Rust evidence covered only a narrower observable, not this complete pinned case"]
fn component_list_case_06_direct_port_expected_red() {
    pending_literal_port(
        r####"TEST_CASE("Component List Artboards Labels", "[component_list]")
{
    auto file = ReadRiveFile("assets/component_list_1.riv");

    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);

    REQUIRE(artboard->find<rive::ArtboardComponentList>("List") != nullptr);
    auto list = artboard->find<rive::ArtboardComponentList>("List");

    artboard->advance(0.0f);

    std::vector<std::string> labels =
        {"ONE", "TWO", "THREE", "THREE", "THREE", "THREE", "TWO", "ONE"};
    for (int i = 0; i < list->artboardCount(); i++)
    {
        auto artboard = list->artboardInstance(i);
        REQUIRE(artboard != nullptr);
        auto label = artboard->find("TextLabel");
        REQUIRE(label->is<rive::Text>());
        REQUIRE(label->as<rive::Text>()->runs()[0]->text() == labels[i]);
    }
}"####,
    );
}

#[test]
#[ignore = "expected-red: prior Rust evidence covered only a narrower observable, not this complete pinned case"]
fn component_list_case_07_direct_port_expected_red() {
    pending_literal_port(
        r####"TEST_CASE("Component list state machine listener", "[component_list]")
{
    auto file = ReadRiveFile("assets/component_list_1.riv");

    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachine("State Machine 1");
    REQUIRE(stateMachine != nullptr);
    rive::StateMachineInstance* stateMachineInstance =
        new rive::StateMachineInstance(stateMachine, artboard.get());
    REQUIRE(stateMachineInstance != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);

    REQUIRE(artboard->find<rive::ArtboardComponentList>("List") != nullptr);
    auto list = artboard->find<rive::ArtboardComponentList>("List");

    artboard->advance(0.0f);

    auto stateMachineInstance1 = list->stateMachineInstance(0);
    auto hoverInput1 = stateMachineInstance1->getBool("Hover");
    REQUIRE(hoverInput1 != nullptr);
    REQUIRE(hoverInput1->value() == false);

    REQUIRE(stateMachineInstance1->hitComponentsCount() == 1);
    // Move over the first shape
    stateMachineInstance->pointerMove(rive::Vec2D(100.0f, 30.0f));
    artboard->advance(0.0f);
    stateMachineInstance->advanceAndApply(0.0f);
    {
        REQUIRE(hoverInput1->value() == true);
    }

    auto stateMachineInstance3 = list->stateMachineInstance(2);
    auto hoverInput3 = stateMachineInstance3->getBool("Hover");
    REQUIRE(hoverInput3 != nullptr);
    REQUIRE(hoverInput3->value() == false);

    REQUIRE(stateMachineInstance3->hitComponentsCount() == 1);
    // Move over the first shape
    stateMachineInstance->pointerMove(rive::Vec2D(100.0f, 150.0f));
    artboard->advance(0.0f);
    stateMachineInstance->advanceAndApply(0.0f);
    {
        REQUIRE(hoverInput3->value() == true);
    }
    delete stateMachineInstance;
}"####,
    );
}

#[test]
#[ignore = "expected-red: prior Rust evidence covered only a narrower observable, not this complete pinned case"]
fn component_list_case_08_direct_port_expected_red() {
    pending_literal_port(
        r####"TEST_CASE("Number To List Artboard Count", "[component_list]")
{
    auto file = ReadRiveFile("assets/component_list_2.riv");

    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);

    REQUIRE(artboard->find<rive::ArtboardComponentList>("List") != nullptr);
    auto list = artboard->find<rive::ArtboardComponentList>("List");

    artboard->advance(0.0f);

    REQUIRE(list->syncStyleChanges() == true);
    REQUIRE(list->artboardCount() == 12);
    REQUIRE(list->layoutNode(13) == nullptr);
    REQUIRE(list->artboardInstance(13) == nullptr);
    REQUIRE(list->stateMachineInstance(13) == nullptr);
    for (int i = 0; i < list->artboardCount(); i++)
    {
        REQUIRE(list->listItem(i) != nullptr);
    }
    auto countProperty = viewModelInstance->propertyValue("ItemCount");
    countProperty->as<rive::ViewModelInstanceNumber>()->propertyValue(6);

    artboard->advance(0.0f);
    REQUIRE(list->artboardCount() == 6);
}"####,
    );
}

#[test]
#[ignore = "expected-red: prior Rust evidence covered only a narrower observable, not this complete pinned case"]
fn component_list_case_09_direct_port_expected_red() {
    pending_literal_port(
        r####"TEST_CASE("Number To List Artboards & State Machines", "[component_list]")
{
    auto file = ReadRiveFile("assets/component_list_2.riv");

    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);

    REQUIRE(artboard->find<rive::ArtboardComponentList>("List") != nullptr);
    auto list = artboard->find<rive::ArtboardComponentList>("List");

    artboard->advance(0.0f);

    for (int i = 0; i < list->artboardCount(); i++)
    {
        auto artboard = list->artboardInstance(i);
        REQUIRE(artboard != nullptr);
        REQUIRE(artboard->name() == "Item");
        auto sm = list->stateMachineInstance(i);
        REQUIRE(sm != nullptr);
        REQUIRE(sm->artboard() == artboard);
    }
}"####,
    );
}

#[test]
#[ignore = "expected-red: prior Rust evidence covered only a narrower observable, not this complete pinned case"]
fn component_list_case_10_direct_port_expected_red() {
    pending_literal_port(
        r####"TEST_CASE("Number To List Artboards Layout Nodes", "[component_list]")
{
    auto file = ReadRiveFile("assets/component_list_2.riv");

    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);

    REQUIRE(artboard->find<rive::ArtboardComponentList>("List") != nullptr);
    auto list = artboard->find<rive::ArtboardComponentList>("List");

    artboard->advance(0.0f);

    for (int i = 0; i < list->artboardCount(); i++)
    {
        auto node = list->layoutNode(i);
        REQUIRE(node != nullptr);
    }
}"####,
    );
}

#[test]
#[ignore = "expected-red: prior Rust evidence covered only a narrower observable, not this complete pinned case"]
fn component_list_case_11_direct_port_expected_red() {
    pending_literal_port(
        r####"TEST_CASE("Number To List Artboards Data Context", "[component_list]")
{
    auto file = ReadRiveFile("assets/component_list_2.riv");

    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);

    REQUIRE(artboard->find<rive::ArtboardComponentList>("List") != nullptr);
    auto list = artboard->find<rive::ArtboardComponentList>("List");

    artboard->advance(0.0f);

    for (int i = 0; i < list->artboardCount(); i++)
    {
        auto artboard = list->artboardInstance(i);
        REQUIRE(artboard != nullptr);
        auto item = list->listItem(i);
        auto vmInstance = item->viewModelInstance();
        auto symbol = vmInstance->propertyValue(rive::SymbolType::itemIndex);
        REQUIRE(symbol != nullptr);
        auto index = symbol->as<rive::ViewModelInstanceSymbolListIndex>()
                         ->propertyValue();
        REQUIRE(index == i);
    }
}"####,
    );
}

#[test]
#[ignore = "expected-red: prior Rust evidence covered only a narrower observable, not this complete pinned case"]
fn component_list_case_12_direct_port_expected_red() {
    pending_literal_port(
        r####"TEST_CASE("Number to List Artboards Labels", "[component_list]")
{
    auto file = ReadRiveFile("assets/component_list_2.riv");

    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);

    REQUIRE(artboard->find<rive::ArtboardComponentList>("List") != nullptr);
    auto list = artboard->find<rive::ArtboardComponentList>("List");

    artboard->advance(0.0f);

    std::vector<std::string> labels =
        {"0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11"};
    for (int i = 0; i < list->artboardCount(); i++)
    {
        auto artboard = list->artboardInstance(i);
        REQUIRE(artboard != nullptr);
        auto label = artboard->find("ItemLabel");
        REQUIRE(label->is<rive::Text>());
        REQUIRE(label->as<rive::Text>()->runs()[0]->text() == labels[i]);
    }
}"####,
    );
}

#[test]
#[ignore = "expected-red: prior Rust evidence covered only a narrower observable, not this complete pinned case"]
fn component_list_case_13_direct_port_expected_red() {
    pending_literal_port(
        r####"TEST_CASE("Component List Virtualized Artboards & State Machines",
          "[component_list]")
{
    auto file = ReadRiveFile("assets/component_list_virtualized.riv");

    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);

    REQUIRE(artboard->find<rive::ArtboardComponentList>("List") != nullptr);
    auto list = artboard->find<rive::ArtboardComponentList>("List");
    artboard->advance(0.0f);

    REQUIRE(list->artboardCount() == 20);
    // Only the first 5 items should be created
    for (int i = 0; i < list->artboardCount(); i++)
    {
        auto artboard = list->artboardInstance(i);
        if (i < 5)
        {
            REQUIRE(artboard != nullptr);
            REQUIRE(artboard->name() == "ItemArtboard");
            auto sm = list->stateMachineInstance(i);
            REQUIRE(sm != nullptr);
            REQUIRE(sm->artboard() == artboard);
        }
        else
        {
            REQUIRE(artboard == nullptr);
            auto sm = list->stateMachineInstance(i);
            REQUIRE(sm == nullptr);
        }
    }
}"####,
    );
}

#[test]
#[ignore = "expected-red: prior Rust evidence covered only a narrower observable, not this complete pinned case"]
fn component_list_case_14_direct_port_expected_red() {
    pending_literal_port(
        r####"TEST_CASE("Component List Virtualized Artboards Layout Bounds",
          "[component_list]")
{
    auto file = ReadRiveFile("assets/component_list_virtualized.riv");

    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);

    REQUIRE(artboard->find<rive::ArtboardComponentList>("List") != nullptr);
    auto list = artboard->find<rive::ArtboardComponentList>("List");
    REQUIRE(list->virtualizationEnabled() == true);

    artboard->advance(0.0f);

    float gap = 10;
    float artboardWidth = 100;
    for (int i = 0; i < list->artboardCount(); i++)
    {
        auto artboard = list->artboardInstance(i);
        if (i < 5)
        {
            REQUIRE(artboard != nullptr);
            auto bounds = list->layoutBoundsForNode(i);
            REQUIRE(bounds.left() == i * (artboardWidth + gap));
        }
        else
        {
            REQUIRE(artboard == nullptr);
        }
    }
}"####,
    );
}

#[test]
#[ignore = "expected-red: prior Rust evidence covered only a narrower observable, not this complete pinned case"]
fn component_list_case_15_direct_port_expected_red() {
    pending_literal_port(
        r####"TEST_CASE("Component List Virtualized Scroll", "[component_list]")
{
    auto file = ReadRiveFile("assets/component_list_virtualized.riv");

    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);

    REQUIRE(artboard->find<rive::ArtboardComponentList>("List") != nullptr);

    REQUIRE(artboard->find<rive::ScrollConstraint>().size() == 1);
    REQUIRE(artboard->find<rive::ScrollConstraint>()[0] != nullptr);
    auto scroll = artboard->find<rive::ScrollConstraint>()[0];

    REQUIRE(scroll->offsetX() == 0);

    artboard->advance(0.0f);

    // scrollIndex
    scroll->setScrollIndex(2);
    REQUIRE(scroll->infinite() == true);
    REQUIRE(scroll->scrollItemCount() == 20);
    REQUIRE(scroll->offsetX() == -220.0f);
    REQUIRE(scroll->clampedOffsetX() == -220.0f);
    REQUIRE(scroll->minOffsetX() == std::numeric_limits<float>::infinity());
    REQUIRE(scroll->maxOffsetX() == -std::numeric_limits<float>::infinity());
    REQUIRE(scroll->offsetY() == 0.0f);
    REQUIRE(scroll->minOffsetY() == 0.0f);
    REQUIRE(scroll->maxOffsetY() == 0.0f);
    REQUIRE(scroll->clampedOffsetY() == 0.0f);
    REQUIRE(scroll->scrollIndex() == 2);
    REQUIRE(scroll->contentWidth() == 2200.0f);
    REQUIRE(scroll->viewportWidth() == 500.0f);
}"####,
    );
}

#[test]
#[ignore = "expected-red: prior Rust evidence covered only a narrower observable, not this complete pinned case"]
fn component_list_case_28_direct_port_expected_red() {
    pending_literal_port(
        r####"TEST_CASE("Component list orderedListIndices default order", "[component_list]")
{
    auto file = ReadRiveFile("assets/component_list_1.riv");

    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);

    artboard->advance(0.0f);

    REQUIRE(artboard->find<rive::ArtboardComponentList>("List") != nullptr);
    auto list = artboard->find<rive::ArtboardComponentList>("List");
    const int n = static_cast<int>(list->artboardCount());
    REQUIRE(n > 0);

    const auto& paintOrder = list->orderedListIndices();
    REQUIRE(static_cast<int>(paintOrder.size()) == n);
    for (int i = 0; i < n; i++)
    {
        REQUIRE(paintOrder[static_cast<size_t>(i)] == i);
    }

    int hitIndex = 0;
    for (auto it = paintOrder.rbegin(); it != paintOrder.rend(); ++it)
    {
        REQUIRE(*it == n - 1 - hitIndex);
        hitIndex++;
    }
}"####,
    );
}

#[test]
#[ignore = "expected-red: prior Rust evidence covered only a narrower observable, not this complete pinned case"]
fn component_list_case_30_direct_port_expected_red() {
    pending_literal_port(
        r####"TEST_CASE("Virtualized list items are clipped by an ancestor layout viewport",
          "[component_list]")
{
    const rive::AABB viewport(100.0f, 100.0f, 300.0f, 300.0f);

    ClipProbeFactory factory;
    auto file =
        ReadRiveFile("assets/component_list_clipped_viewport.riv", &factory);

    auto artboard = file->artboardNamed("Main");
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);
    artboard->advance(0.0f);

    auto list = artboard->find<rive::ArtboardComponentList>("List");
    REQUIRE(list != nullptr);
    REQUIRE(list->artboardCount() == 6);
    REQUIRE(list->virtualizationEnabled() == true);
    // Only items intersecting the viewport are mounted.
    REQUIRE(countMounted(list) == 4);

    ClipProbeRenderer renderer;
    artboard->draw(&renderer);

    // Artboard background, four item fills, overlay fill.
    REQUIRE(renderer.draws.size() == 6);

    // Every item fill draws with the viewport clip active.
    for (int i = 1; i <= 4; i++)
    {
        auto& item = renderer.draws[i];
        REQUIRE(item.hasClip);
        REQUIRE(item.clip.minX == viewport.minX);
        REQUIRE(item.clip.minY == viewport.minY);
        REQUIRE(item.clip.maxX == viewport.maxX);
        REQUIRE(item.clip.maxY == viewport.maxY);
    }
    // The last mounted item straddles the viewport bottom; without the clip
    // it would paint outside the viewport.
    REQUIRE(renderer.draws[4].bounds.maxY > viewport.maxY);

    // The overlay is outside the viewport subtree and must not inherit its
    // clip; it draws after (above) the items.
    auto& overlay = renderer.draws[5];
    REQUIRE(overlay.bounds.minY == 360.0f);
    if (overlay.hasClip)
    {
        // Any clip it does carry (the artboard's) is wider than the viewport.
        REQUIRE(overlay.clip.maxY > viewport.maxY);
    }

    // Scrolling moves the virtualization window and keeps items clipped.
    auto scrolls = artboard->find<rive::ScrollConstraint>();
    REQUIRE(scrolls.size() == 1);
    auto scroll = scrolls[0];
    scroll->setScrollIndex(2);
    artboard->advance(0.0f);

    REQUIRE(list->artboardInstance(0) == nullptr);
    REQUIRE(list->artboardInstance(5) != nullptr);

    ClipProbeRenderer scrolled;
    artboard->draw(&scrolled);
    REQUIRE(scrolled.draws.size() == 6);
    for (int i = 1; i <= 4; i++)
    {
        REQUIRE(scrolled.draws[i].hasClip);
        REQUIRE(scrolled.draws[i].clip.minY == viewport.minY);
        REQUIRE(scrolled.draws[i].clip.maxY == viewport.maxY);
    }
    // First visible item is item 2, flush with the viewport top.
    REQUIRE(scrolled.draws[1].bounds.minY == viewport.minY);
}"####,
    );
}
