#import <Foundation/Foundation.h>
#include <math.h>
#include <stdint.h>

extern void *nux_video_apple_open(const char *, uint64_t);
extern void nux_video_apple_close(void *);
extern void nux_video_apple_action(void *, int, double, uint64_t);
extern int nux_video_apple_poll(void *, uint64_t *, double *, uint32_t *, uint32_t *);
extern bool nux_video_apple_copy_rgba(void *, uint8_t *, size_t);
extern void nux_video_apple_pump(double);

static bool expect_no_duplicate(void *player) {
    double deadline = NSDate.timeIntervalSinceReferenceDate + .1;
    while (NSDate.timeIntervalSinceReferenceDate < deadline) {
        uint64_t generation; double pts; uint32_t width, height;
        int event = nux_video_apple_poll(player, &generation, &pts, &width, &height);
        if (event == 4 || event == 5) {
            printf("FAIL: paused frame emitted repeatedly\n");
            return false;
        }
        nux_video_apple_pump(.005);
    }
    return true;
}

static bool expect_frame(void *player, uint64_t expected_generation, double target, bool blue) {
    double deadline = NSDate.timeIntervalSinceReferenceDate + 3;
    while (NSDate.timeIntervalSinceReferenceDate < deadline) {
        uint64_t generation = 0;
        double pts = 0;
        uint32_t width = 0, height = 0;
        int event = nux_video_apple_poll(player, &generation, &pts, &width, &height);
        if (event < 0) return false;
        if (event == 4 || event == 5) {
            if (!width || !height || width > 4096 || height > 4096) return false;
            size_t length = (size_t)width * height * 4;
            uint8_t *rgba = malloc(length);
            bool copied = nux_video_apple_copy_rgba(player, rgba, length);
            bool color = copied && (blue ? rgba[2] > 240 && rgba[0] < 15 : rgba[0] > 240 && rgba[2] < 15);
            free(rgba);
            bool selected = expected_generation == 1 ? event == 4 : event == 5;
            double expected_pts = target >= 2 ? 2.0 - 1.0 / 30.0 : target;
            bool pass = selected && generation == expected_generation && fabs(pts - expected_pts) <= 0.000001 && color;
            printf("generation=%llu target=%.6f pts=%.6f pixels=%s %s\n", generation, target, pts, color ? "correct" : "wrong", pass ? "PASS" : "FAIL");
            return pass && expect_no_duplicate(player);
        }
        nux_video_apple_pump(.005);
    }
    printf("generation=%llu target=%.6f FAIL: no decoded frame\n", expected_generation, target);
    return false;
}

int main(int argc, char **argv) {
    @autoreleasepool {
        if (argc != 2) return 2;
        void *player = nux_video_apple_open(argv[1], 1);
        if (!player) return 2;
        int failures = !expect_frame(player, 1, 0, false);
        // The output has already served these decoded frames. A new seek owner
        // still needs its own presentation acknowledgement at zero and EOF.
        double targets[] = {0, .4, 2.0 - 1.0 / 30.0, 2.022, .4};
        for (int i = 0; i < 5; ++i) {
            nux_video_apple_action(player, 2, targets[i], i + 2);
            failures += !expect_frame(player, i + 2, targets[i], targets[i] > 1);
        }
        // An obsolete completion cannot acknowledge the replacement's frame.
        nux_video_apple_action(player, 2, 0, 7);
        nux_video_apple_action(player, 2, 2.0 - 1.0 / 30.0, 8);
        failures += !expect_frame(player, 8, 2.0 - 1.0 / 30.0, true);
        nux_video_apple_close(player);
        return failures ? 1 : 0;
    }
}
