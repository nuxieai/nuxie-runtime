#import <UIKit/UIKit.h>
extern void *nux_video_proof_open(const char *source, bool audible, bool embedded, bool pool, bool sync);
extern size_t nux_video_proof_caption(const void *proof, unsigned char *output, size_t capacity);
extern size_t nux_video_proof_preview(const void *proof, unsigned char *output, size_t capacity);
extern int nux_video_proof_tick(void *proof);
extern int nux_video_proof_lifecycle_status(const void *proof);
extern int nux_video_proof_backgrounded(void *proof, bool suspended);
extern void nux_video_proof_close(void *proof);

extern void *nux_video_benchmark_open(const char *source);
extern int nux_video_benchmark_tick(void *proof);
extern size_t nux_video_benchmark_copy(const void *proof, unsigned char *output, size_t capacity, bool preview);
extern void nux_video_benchmark_close(void *proof);

@interface VideoProofController : UIViewController
@property UIImageView *preview;
@property UILabel *caption;
@property UILabel *result;
@property NSTimer *timer;
@property void *proof;
@property void *benchmark;
@property BOOL redCaption;
@property BOOL blueCaption;
@property BOOL lifecycleRequested;
@property BOOL applicationSuspended;
@end
@implementation VideoProofController
- (void)viewDidLoad {
    [super viewDidLoad];
    self.view.backgroundColor = UIColor.blackColor;
    self.preview = [UIImageView new];
    self.preview.contentMode = UIViewContentModeScaleAspectFit;
    self.preview.layer.magnificationFilter = kCAFilterNearest;
    self.preview.isAccessibilityElement = YES;
    self.preview.accessibilityLabel = @"Runtime video preview with green scene overlay";
    self.caption = [UILabel new];
    self.caption.textColor = UIColor.whiteColor;
    self.caption.textAlignment = NSTextAlignmentCenter;
    self.caption.font = [UIFont preferredFontForTextStyle:UIFontTextStyleTitle2];
    self.result = [UILabel new];
    self.result.textColor = UIColor.whiteColor;
    self.result.numberOfLines = 0;
    self.result.textAlignment = NSTextAlignmentCenter;
    self.result.font = [UIFont preferredFontForTextStyle:UIFontTextStyleBody];
    UIButton *replay = [UIButton buttonWithType:UIButtonTypeSystem];
    [replay setTitle:@"Run again" forState:UIControlStateNormal];
    [replay addTarget:self action:@selector(runProof) forControlEvents:UIControlEventTouchUpInside];
    UIStackView *stack = [[UIStackView alloc] initWithArrangedSubviews:@[self.preview, self.caption, self.result, replay]];
    stack.axis = UILayoutConstraintAxisVertical;
    stack.spacing = 20;
    stack.translatesAutoresizingMaskIntoConstraints = NO;
    [self.view addSubview:stack];
    [NSLayoutConstraint activateConstraints:@[
        [stack.leadingAnchor constraintEqualToAnchor:self.view.safeAreaLayoutGuide.leadingAnchor constant:24],
        [stack.trailingAnchor constraintEqualToAnchor:self.view.safeAreaLayoutGuide.trailingAnchor constant:-24],
        [stack.centerYAnchor constraintEqualToAnchor:self.view.safeAreaLayoutGuide.centerYAnchor],
        [self.preview.heightAnchor constraintEqualToAnchor:self.preview.widthAnchor multiplier:0.5],
        [self.caption.heightAnchor constraintGreaterThanOrEqualToConstant:32]
    ]];
    [self runProof];
}
- (void)setBackgrounded:(BOOL)suspended {
    // The initial didBecomeActive callback is not a resume. Forward only
    // actual app transitions so it cannot clear an independent pause probe.
    if (self.applicationSuspended == suspended) return;
    self.applicationSuspended = suspended;
    if (self.benchmark) {
        if (suspended) {
            self.result.text = @"FAIL: 720p benchmark interrupted by app suspension";
            NSLog(@"NUX_VIDEO_BENCHMARK status=-1 app suspended");
            [self stopProof];
        }
        return;
    }
    if (nux_video_proof_backgrounded(self.proof, suspended) != 0) {
        self.result.text = @"FAIL: lifecycle action rejected";
        [self stopProof];
        return;
    }
    self.timer.fireDate = suspended ? NSDate.distantFuture : NSDate.date;
}
- (void)stopProof {
    [self.timer invalidate]; self.timer = nil;
    if (self.proof) { nux_video_proof_close(self.proof); self.proof = NULL; }
    if (self.benchmark) { nux_video_benchmark_close(self.benchmark); self.benchmark = NULL; }
}
- (void)runProof {
    [self stopProof];
    self.redCaption = NO; self.blueCaption = NO; self.lifecycleRequested = NO;
    self.preview.image = nil; self.caption.text = @"";
    self.result.text = @"Checking video, captions and decoder reopening…";
    NSArray *arguments = NSProcessInfo.processInfo.arguments;
    if ([arguments containsObject:@"--benchmark"]) {
        self.result.text = @"Running optimized 720p decode and Metal composition benchmark (32 seconds)…";
        NSString *benchmarkPath = [[NSBundle mainBundle] pathForResource:@"red-blue-720p" ofType:@"mp4"];
        self.benchmark = nux_video_benchmark_open(benchmarkPath.UTF8String);
        if (!self.benchmark) { self.result.text = @"FAIL: benchmark initialization"; return; }
        __weak VideoProofController *weakSelf = self;
        self.timer = [NSTimer scheduledTimerWithTimeInterval:1.0/60.0 repeats:YES block:^(NSTimer *timer) {
            [weakSelf advance];
        }];
        return;
    }
    NSString *path = [[NSBundle mainBundle] pathForResource:@"red-blue-audio" ofType:@"mp4"];
    self.proof = nux_video_proof_open(path.UTF8String, [arguments containsObject:@"--audible"], [arguments containsObject:@"--embedded"], [arguments containsObject:@"--pool"], [arguments containsObject:@"--sync"]);
    __weak VideoProofController *weakSelf = self;
    self.timer = [NSTimer scheduledTimerWithTimeInterval:1.0/60.0 repeats:YES block:^(NSTimer *timer) {
        [weakSelf advance];
    }];
}
- (void)advance {
    BOOL benchmark = self.benchmark != NULL;
    int status = benchmark ? nux_video_benchmark_tick(self.benchmark) : nux_video_proof_tick(self.proof);
    unsigned char captionBytes[256];
    size_t count = benchmark ? 0 : nux_video_proof_caption(self.proof, captionBytes, sizeof(captionBytes));
    NSString *caption = [[NSString alloc] initWithBytes:captionBytes length:count encoding:NSUTF8StringEncoding];
    self.redCaption |= [caption isEqualToString:@"Red scene"];
    self.blueCaption |= [caption isEqualToString:@"Blue scene"];
    self.caption.text = caption ?: @"";
    unsigned char pixels[64 * 32 * 4];
    count = benchmark ? nux_video_benchmark_copy(self.benchmark, pixels, sizeof(pixels), true) : nux_video_proof_preview(self.proof, pixels, sizeof(pixels));
    if (count == sizeof(pixels)) {
        NSData *data = [NSData dataWithBytes:pixels length:count];
        CGDataProviderRef provider = CGDataProviderCreateWithCFData((__bridge CFDataRef)data);
        CGColorSpaceRef color = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
        CGImageRef image = CGImageCreate(64, 32, 8, 32, 64 * 4, color,
            kCGBitmapByteOrder32Big | kCGImageAlphaPremultipliedLast, provider, NULL, NO, kCGRenderingIntentDefault);
        self.preview.image = [UIImage imageWithCGImage:image];
        CGImageRelease(image); CGColorSpaceRelease(color); CGDataProviderRelease(provider);
    }
    if (benchmark) {
        if (status == 0) return;
        unsigned char bytes[2048];
        size_t length = nux_video_benchmark_copy(self.benchmark, bytes, sizeof(bytes), false);
        NSString *metrics = [[NSString alloc] initWithBytes:bytes length:length encoding:NSUTF8StringEncoding];
        self.result.text = [NSString stringWithFormat:@"%@: 720p Metal benchmark\n%@", status == 1 ? @"PASS" : @"FAIL", metrics ?: @"No metrics"];
        [self.result.text writeToFile:[NSHomeDirectory() stringByAppendingPathComponent:@"Documents/video-proof-result.txt"] atomically:YES encoding:NSUTF8StringEncoding error:nil];
        NSLog(@"NUX_VIDEO_BENCHMARK status=%d %@", status, self.result.text);
        [self stopProof];
        return;
    }
    BOOL lifecycle = [NSProcessInfo.processInfo.arguments containsObject:@"--lifecycle"];
    if (status == 0 && lifecycle && !self.lifecycleRequested && nux_video_proof_lifecycle_status(self.proof) == 1) {
        self.lifecycleRequested = YES;
        NSLog(@"NUX_VIDEO_OS_LIFECYCLE opening app Settings; return to resume proof");
        [UIApplication.sharedApplication openURL:[NSURL URLWithString:UIApplicationOpenSettingsURLString] options:@{} completionHandler:^(BOOL success) {
            if (!success) { self.result.text = @"FAIL: could not background into app Settings"; [self stopProof]; }
        }];
    }
    if (status == 0) return;
    if (status == 1 && lifecycle && nux_video_proof_lifecycle_status(self.proof) != 2) status = -1;
    if (status == 1 && !(self.redCaption && self.blueCaption)) status = -1;
    [self stopProof];
    BOOL embedded = [NSProcessInfo.processInfo.arguments containsObject:@"--embedded"];
    BOOL audible = [NSProcessInfo.processInfo.arguments containsObject:@"--audible"];
    NSString *result = status == 1 ? @"PASS: AVFoundation → .nux → Metal\nPixels, captions, seek, two loops, reclamation/reopen, completion, disposal" : @"FAIL: inspect device console";
    if (status == 1 && [NSProcessInfo.processInfo.arguments containsObject:@"--sync"]) {
        result = @"PASS: native clock synchronization\nInjected offset, measured convergence, pixels, captions, disposal";
    }
    if ([NSProcessInfo.processInfo.arguments containsObject:@"--pool"] && ![NSProcessInfo.processInfo.arguments containsObject:@"--sync"]) {
        result = [result stringByAppendingString:@"\nTwo-player priority/concurrency qualification"];
    }
    result = [result stringByAppendingFormat:@"\nembedded=%@; %@\nDecode acceleration unknown", embedded ? @"true" : @"false", audible ? @"AAC enabled at 20%" : @"muted AAC"];
    if (lifecycle && status == 1) result = [result stringByAppendingString:@"\nOS background/resume qualified"];
    self.result.text = result;
    [result writeToFile:[NSHomeDirectory() stringByAppendingPathComponent:@"Documents/video-proof-result.txt"] atomically:YES encoding:NSUTF8StringEncoding error:nil];
    NSLog(@"NUX_VIDEO_PROOF status=%d %@", status, result);
}
- (void)dealloc { [self stopProof]; }
@end
@interface VideoProofApp : UIResponder <UIApplicationDelegate>
@property UIWindow *window;
@end
@implementation VideoProofApp
- (void)applicationWillResignActive:(UIApplication *)application {
    [(VideoProofController *)self.window.rootViewController setBackgrounded:YES];
}
- (void)applicationDidBecomeActive:(UIApplication *)application {
    [(VideoProofController *)self.window.rootViewController setBackgrounded:NO];
}
- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)options {
    self.window = [[UIWindow alloc] initWithFrame:UIScreen.mainScreen.bounds];
    self.window.rootViewController = [VideoProofController new];
    [self.window makeKeyAndVisible];
    return YES;
}
@end
int main(int argc,char **argv) { @autoreleasepool { return UIApplicationMain(argc,argv,nil,NSStringFromClass(VideoProofApp.class)); } }
