// Independent ground-truth oracle for the Luau parser port.
//
// Parses a .luau file with the REAL Luau C++ parser (`luau/Ast/src/Parser.cpp`)
// built standalone, then walks the resulting AST depth-first and emits a
// canonical, line-oriented structural dump. The Rust port's `parser_oracle_test`
// produces the byte-identical dump from the same fixtures; any divergence is a
// fidelity bug in the port.
//
// Dump grammar (one node per line, 2 spaces of indent per depth level):
//
//   <indent><Kind> <bL>:<bC>-<eL>:<eC>[ key=val ...]
//
// Conventions shared verbatim with the Rust dumper:
//   * Children are emitted in SOURCE order (left-to-right as they appear).
//   * Identifier / string payloads are lowercase hex of their raw bytes
//     (`n=<hex>`), so delimiters and non-ASCII never corrupt the stream.
//   * A number literal's payload is the 64-bit IEEE-754 hex of its value
//     (`v=<016x>`), so it is exact.
//   * Enum ops (binary/unary/compound) are printed as their raw enum integer.
//   * An AST node kind the dumper does not cover prints `!!UNCOVERED idx=<n>`
//     and stops descending, so a gap is loud rather than silently matching.
//
// Regenerate goldens with testdata/regen_parse_golden.sh.

#include "Luau/Ast.h"
#include "Luau/Parser.h"
#include "Luau/Allocator.h"
#include <cstdio>
#include <cstring>
#include <string>

using namespace Luau;

static void indent(int depth)
{
    for (int i = 0; i < depth; ++i)
        printf("  ");
}

static void printHex(const char* p, size_t n)
{
    for (size_t i = 0; i < n; ++i)
        printf("%02x", (unsigned char)p[i]);
}

static void printName(const char* label, const AstName& name)
{
    printf(" %s=", label);
    if (name.value)
        printHex(name.value, strlen(name.value));
    else
        printf("-");
}

static void printLoc(const Location& loc)
{
    printf(" %u:%u-%u:%u", loc.begin.line, loc.begin.column, loc.end.line, loc.end.column);
}

static void printLocal(const char* label, AstLocal* l)
{
    printf(" %s=", label);
    if (l && l->name.value)
        printHex(l->name.value, strlen(l->name.value));
    else
        printf("-");
}

static void dumpNode(AstNode* node, int depth);

static void dumpExprList(const AstArray<AstExpr*>& list, int depth)
{
    for (size_t i = 0; i < list.size; ++i)
        dumpNode(list.data[i], depth);
}

static void dumpStatList(const AstArray<AstStat*>& list, int depth)
{
    for (size_t i = 0; i < list.size; ++i)
        dumpNode(list.data[i], depth);
}

// Emits "<Kind> <loc>" then leaves the line open for the caller to append
// payload key=val pairs before the newline.
static void head(const char* kind, AstNode* node, int depth)
{
    indent(depth);
    printf("%s", kind);
    printLoc(node->location);
}

static void dumpNode(AstNode* node, int depth)
{
    if (!node)
    {
        indent(depth);
        printf("<null>\n");
        return;
    }

    // ---- expressions ----
    if (auto* e = node->as<AstExprGroup>())
    {
        head("Group", e, depth);
        printf("\n");
        dumpNode(e->expr, depth + 1);
        return;
    }
    if (auto* e = node->as<AstExprConstantNil>())
    {
        head("Nil", e, depth);
        printf("\n");
        return;
    }
    if (auto* e = node->as<AstExprConstantBool>())
    {
        head("Bool", e, depth);
        printf(" v=%d\n", e->value ? 1 : 0);
        return;
    }
    if (auto* e = node->as<AstExprConstantNumber>())
    {
        head("Number", e, depth);
        uint64_t bits;
        memcpy(&bits, &e->value, 8);
        printf(" v=%016llx pr=%d\n", (unsigned long long)bits, (int)e->parseResult);
        return;
    }
    if (auto* e = node->as<AstExprConstantString>())
    {
        head("String", e, depth);
        printf(" v=");
        printHex(e->value.data, e->value.size);
        printf(" q=%d\n", (int)e->quoteStyle);
        return;
    }
    if (auto* e = node->as<AstExprLocal>())
    {
        head("Local", e, depth);
        printLocal("l", e->local);
        printf(" up=%d\n", e->upvalue ? 1 : 0);
        return;
    }
    if (auto* e = node->as<AstExprGlobal>())
    {
        head("Global", e, depth);
        printName("n", e->name);
        printf("\n");
        return;
    }
    if (auto* e = node->as<AstExprVarargs>())
    {
        head("Varargs", e, depth);
        printf("\n");
        return;
    }
    if (auto* e = node->as<AstExprCall>())
    {
        head("Call", e, depth);
        printf(" self=%d nargs=%zu\n", e->self ? 1 : 0, e->args.size);
        dumpNode(e->func, depth + 1);
        dumpExprList(e->args, depth + 1);
        return;
    }
    if (auto* e = node->as<AstExprIndexName>())
    {
        head("IndexName", e, depth);
        printName("i", e->index);
        printf(" op=%c\n", e->op);
        dumpNode(e->expr, depth + 1);
        return;
    }
    if (auto* e = node->as<AstExprIndexExpr>())
    {
        head("IndexExpr", e, depth);
        printf("\n");
        dumpNode(e->expr, depth + 1);
        dumpNode(e->index, depth + 1);
        return;
    }
    if (auto* e = node->as<AstExprFunction>())
    {
        head("Function", e, depth);
        printf(" vararg=%d nargs=%zu", e->vararg ? 1 : 0, e->args.size);
        printLocal("self", e->self);
        printName("debug", e->debugname);
        printf("\n");
        dumpNode(e->body, depth + 1);
        return;
    }
    if (auto* e = node->as<AstExprTable>())
    {
        head("Table", e, depth);
        printf(" nitems=%zu\n", e->items.size);
        for (size_t i = 0; i < e->items.size; ++i)
        {
            indent(depth + 1);
            printf("Item kind=%d\n", (int)e->items.data[i].kind);
            if (e->items.data[i].key)
                dumpNode(e->items.data[i].key, depth + 2);
            dumpNode(e->items.data[i].value, depth + 2);
        }
        return;
    }
    if (auto* e = node->as<AstExprUnary>())
    {
        head("Unary", e, depth);
        printf(" op=%d\n", (int)e->op);
        dumpNode(e->expr, depth + 1);
        return;
    }
    if (auto* e = node->as<AstExprBinary>())
    {
        head("Binary", e, depth);
        printf(" op=%d\n", (int)e->op);
        dumpNode(e->left, depth + 1);
        dumpNode(e->right, depth + 1);
        return;
    }
    if (auto* e = node->as<AstExprIfElse>())
    {
        head("IfElseExpr", e, depth);
        printf(" hasThen=%d hasElse=%d\n", e->hasThen ? 1 : 0, e->hasElse ? 1 : 0);
        dumpNode(e->condition, depth + 1);
        dumpNode(e->trueExpr, depth + 1);
        dumpNode(e->falseExpr, depth + 1);
        return;
    }

    // ---- statements ----
    if (auto* s = node->as<AstStatBlock>())
    {
        head("Block", s, depth);
        printf(" hasEnd=%d\n", s->hasEnd ? 1 : 0);
        dumpStatList(s->body, depth + 1);
        return;
    }
    if (auto* s = node->as<AstStatIf>())
    {
        head("If", s, depth);
        printf("\n");
        dumpNode(s->condition, depth + 1);
        dumpNode(s->thenbody, depth + 1);
        if (s->elsebody)
            dumpNode(s->elsebody, depth + 1);
        return;
    }
    if (auto* s = node->as<AstStatWhile>())
    {
        head("While", s, depth);
        printf(" hasDo=%d\n", s->hasDo ? 1 : 0);
        dumpNode(s->condition, depth + 1);
        dumpNode(s->body, depth + 1);
        return;
    }
    if (auto* s = node->as<AstStatRepeat>())
    {
        head("Repeat", s, depth);
        printf("\n");
        dumpNode(s->body, depth + 1);
        dumpNode(s->condition, depth + 1);
        return;
    }
    if (auto* s = node->as<AstStatBreak>())
    {
        head("Break", s, depth);
        printf("\n");
        return;
    }
    if (auto* s = node->as<AstStatContinue>())
    {
        head("Continue", s, depth);
        printf("\n");
        return;
    }
    if (auto* s = node->as<AstStatReturn>())
    {
        head("Return", s, depth);
        printf(" n=%zu\n", s->list.size);
        dumpExprList(s->list, depth + 1);
        return;
    }
    if (auto* s = node->as<AstStatExpr>())
    {
        head("ExprStat", s, depth);
        printf("\n");
        dumpNode(s->expr, depth + 1);
        return;
    }
    if (auto* s = node->as<AstStatLocal>())
    {
        head("LocalStat", s, depth);
        printf(" nvars=%zu nvalues=%zu\n", s->vars.size, s->values.size);
        for (size_t i = 0; i < s->vars.size; ++i)
        {
            indent(depth + 1);
            printf("Var");
            printLocal("n", s->vars.data[i]);
            printf("\n");
        }
        dumpExprList(s->values, depth + 1);
        return;
    }
    if (auto* s = node->as<AstStatFor>())
    {
        head("For", s, depth);
        printLocal("var", s->var);
        printf(" hasDo=%d\n", s->hasDo ? 1 : 0);
        dumpNode(s->from, depth + 1);
        dumpNode(s->to, depth + 1);
        if (s->step)
            dumpNode(s->step, depth + 1);
        dumpNode(s->body, depth + 1);
        return;
    }
    if (auto* s = node->as<AstStatForIn>())
    {
        head("ForIn", s, depth);
        printf(" nvars=%zu nvalues=%zu hasIn=%d hasDo=%d\n", s->vars.size, s->values.size, s->hasIn ? 1 : 0,
            s->hasDo ? 1 : 0);
        for (size_t i = 0; i < s->vars.size; ++i)
        {
            indent(depth + 1);
            printf("Var");
            printLocal("n", s->vars.data[i]);
            printf("\n");
        }
        dumpExprList(s->values, depth + 1);
        dumpNode(s->body, depth + 1);
        return;
    }
    if (auto* s = node->as<AstStatAssign>())
    {
        head("Assign", s, depth);
        printf(" nvars=%zu nvalues=%zu\n", s->vars.size, s->values.size);
        dumpExprList(s->vars, depth + 1);
        dumpExprList(s->values, depth + 1);
        return;
    }
    if (auto* s = node->as<AstStatCompoundAssign>())
    {
        head("CompoundAssign", s, depth);
        printf(" op=%d\n", (int)s->op);
        dumpNode(s->var, depth + 1);
        dumpNode(s->value, depth + 1);
        return;
    }
    if (auto* s = node->as<AstStatFunction>())
    {
        head("FunctionStat", s, depth);
        printf("\n");
        dumpNode(s->name, depth + 1);
        dumpNode(s->func, depth + 1);
        return;
    }
    if (auto* s = node->as<AstStatLocalFunction>())
    {
        head("LocalFunction", s, depth);
        printLocal("name", s->name);
        printf("\n");
        dumpNode(s->func, depth + 1);
        return;
    }
    if (auto* s = node->as<AstStatError>())
    {
        head("StatError", s, depth);
        printf("\n");
        return;
    }
    if (auto* e = node->as<AstExprError>())
    {
        head("ExprError", e, depth);
        printf("\n");
        return;
    }

    // Loud, non-matching marker for any kind we have not taught the dumper yet.
    indent(depth);
    printf("!!UNCOVERED idx=%d\n", node->classIndex);
}

int main(int argc, char** argv)
{
    if (argc < 2)
        return 2;
    FILE* f = fopen(argv[1], "rb");
    if (!f)
        return 2;
    std::string s;
    char b[4096];
    size_t r;
    while ((r = fread(b, 1, sizeof(b), f)) > 0)
        s.append(b, r);
    fclose(f);

    Allocator allocator;
    AstNameTable names(allocator);
    ParseOptions options;
    ParseResult result = Parser::parse(s.data(), s.size(), names, allocator, options);

    for (const ParseError& e : result.errors)
        fprintf(stderr, "parse error %u:%u: %s\n", e.getLocation().begin.line, e.getLocation().begin.column,
            e.what());

    dumpNode(result.root, 0);
    return result.errors.empty() ? 0 : 1;
}
