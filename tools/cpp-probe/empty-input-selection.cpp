// Diagnostic reproducer, not part of the normal test/build targets.
// Unmodified upstream 9ed5b5168d95aab07e873db341fb65613d317cfc faults when
// selectionRects receives a shape with no ordered lines. This isolates the
// cursor-side assumption; it is not an end-to-end font/shaper oracle.
#include "rive/text/cursor.hpp"
#include "rive/text/fully_shaped_text.hpp"
#include <cstdio>

int main()
{
    rive::FullyShapedText empty;
    std::vector<rive::AABB> rects;
    std::fprintf(stderr, "ordered-lines=%zu\n", empty.orderedLines().size());
    rive::Cursor::zero().selectionRects(rects, empty);
    std::fprintf(stderr, "selection-rects=%zu\n", rects.size());
}
