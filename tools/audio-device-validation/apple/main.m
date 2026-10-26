#import <UIKit/UIKit.h>
#import <AVFoundation/AVFoundation.h>
#import <stdatomic.h>
#import <stdbool.h>
#import <stdint.h>
#import <string.h>
#import <stdio.h>
#import <math.h>

extern void *nux_audio_probe_new(uint32_t, uint32_t, uint32_t);
extern uint64_t nux_audio_probe_pull(void *, float *, uint32_t);
extern void nux_audio_probe_set_phase(void *, uint32_t);
extern char *nux_audio_probe_report_json(void *);
extern void nux_audio_probe_free_json(char *);
extern void nux_audio_probe_destroy(void *);

typedef struct {
    void *probe;
    float interleavedScratch[4096 * 2];
    _Atomic bool enabled;
    _Atomic uint64_t callbacks;
    _Atomic uint64_t active;
    _Atomic uint64_t invalidSamples;
    _Atomic uint64_t wrongLayouts;
    _Atomic uint64_t callbackFramesMin;
    _Atomic uint64_t callbackFramesMax;
} AudioContext;

@interface AudioContextOwner : NSObject { @public AudioContext state; }
@end
@implementation AudioContextOwner
@end

@interface AudioValidationDelegate : UIResponder <UIApplicationDelegate>
@property(nonatomic, strong) UIWindow *window;
@property(nonatomic, strong) UILabel *label;
@property(nonatomic, strong) AVAudioEngine *engine;
@property(nonatomic, strong) AVAudioSourceNode *source;
@property(nonatomic) AudioContext *context;
@property(nonatomic, strong) AudioContextOwner *contextOwner;
@property(nonatomic) uint32_t phase;
@property(nonatomic, copy) NSString *oldCategory;
@property(nonatomic, copy) NSString *oldMode;
@property(nonatomic) AVAudioSessionCategoryOptions oldOptions;
@end

@implementation AudioValidationDelegate
- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)options {
    self.window = [[UIWindow alloc] initWithFrame:UIScreen.mainScreen.bounds];
    UIViewController *controller = [UIViewController new];
    controller.view.backgroundColor = UIColor.systemBackgroundColor;
    self.label = [[UILabel alloc] initWithFrame:CGRectInset(self.window.bounds, 24, 100)];
    self.label.numberOfLines = 0;
    self.label.text = @"Nuxie audio validation\nStarting low-volume audio…";
    [controller.view addSubview:self.label];
    self.window.rootViewController = controller;
    [self.window makeKeyAndVisible];
    application.idleTimerDisabled = YES;
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, NSEC_PER_SEC), dispatch_get_main_queue(), ^{ [self start]; });
    return YES;
}
- (void)fail:(NSString *)message {
    NSLog(@"NUX_AUDIO_IOS_FAILURE %@", message);
    self.label.text = [@"Audio validation failed\n" stringByAppendingString:message];
    [self finish];
}
- (void)start {
    AVAudioSession *session = AVAudioSession.sharedInstance;
    self.oldCategory = session.category;
    self.oldMode = session.mode;
    self.oldOptions = session.categoryOptions;
    NSError *error = nil;
    if (![session setCategory:AVAudioSessionCategoryPlayback mode:AVAudioSessionModeDefault
                      options:AVAudioSessionCategoryOptionMixWithOthers error:&error] ||
        ![session setPreferredSampleRate:48000 error:&error] ||
        ![session setPreferredIOBufferDuration:256.0 / 48000.0 error:&error] ||
        ![session setActive:YES error:&error]) {
        [self fail:[NSString stringWithFormat:@"audio session: %@", error]]; return;
    }
    uint32_t rate = (uint32_t)llround(session.sampleRate);
    AudioContextOwner *owner = [AudioContextOwner new];
    self.contextOwner = owner;
    AudioContext *context = &owner->state;
    self.context = context;
    context->probe = nux_audio_probe_new(rate, 2, 256);
    if (!context->probe) { [self fail:@"probe creation failed"]; return; }
    atomic_init(&context->enabled, true);
    atomic_init(&context->callbacks, 0);
    atomic_init(&context->active, 0);
    atomic_init(&context->invalidSamples, 0);
    atomic_init(&context->wrongLayouts, 0);
    atomic_init(&context->callbackFramesMax, 0);
    atomic_init(&context->callbackFramesMin, UINT64_MAX);
    AVAudioFormat *format = [[AVAudioFormat alloc] initWithCommonFormat:AVAudioPCMFormatFloat32
                                                           sampleRate:rate channels:2 interleaved:NO];
    self.engine = [AVAudioEngine new];
    self.source = [[AVAudioSourceNode alloc] initWithFormat:format renderBlock:
        ^OSStatus(BOOL *isSilence, const AudioTimeStamp *timestamp, AVAudioFrameCount count, AudioBufferList *buffers) {
            AudioContext *context = &owner->state;
            atomic_fetch_add(&context->active, 1);
            for (UInt32 i = 0; i < buffers->mNumberBuffers; ++i) {
                if (buffers->mBuffers[i].mData) memset(buffers->mBuffers[i].mData, 0, buffers->mBuffers[i].mDataByteSize);
            }
            if (atomic_load(&context->enabled)) {
                if (buffers->mNumberBuffers != 2 || buffers->mBuffers[0].mNumberChannels != 1 ||
                    buffers->mBuffers[1].mNumberChannels != 1 || !buffers->mBuffers[0].mData ||
                    !buffers->mBuffers[1].mData || buffers->mBuffers[0].mDataByteSize < count * sizeof(float) ||
                    buffers->mBuffers[1].mDataByteSize < count * sizeof(float)) {
                    atomic_fetch_add(&context->wrongLayouts, 1);
                } else {
                    float *left = buffers->mBuffers[0].mData;
                    float *right = buffers->mBuffers[1].mData;
                    uint64_t invalid = 0;
                    // AVAudioEngine requires planar PCM. The runtime stays interleaved;
                    // adapt using storage allocated before callbacks, never on the audio thread.
                    for (uint32_t offset = 0; offset < count;) {
                        uint32_t chunk = MIN(4096, count - offset);
                        nux_audio_probe_pull(context->probe, context->interleavedScratch, chunk);
                        for (uint32_t i = 0; i < chunk; ++i) {
                            float l = context->interleavedScratch[i * 2];
                            float r = context->interleavedScratch[i * 2 + 1];
                            invalid += !isfinite(l) || fabsf(l) > 1.0f;
                            invalid += !isfinite(r) || fabsf(r) > 1.0f;
                            left[offset + i] = l;
                            right[offset + i] = r;
                        }
                        offset += chunk;
                    }
                    atomic_fetch_add(&context->invalidSamples, invalid);
                    atomic_fetch_add(&context->callbacks, 1);
                    uint64_t min = atomic_load(&context->callbackFramesMin), max = atomic_load(&context->callbackFramesMax);
                    if (count < min) atomic_store(&context->callbackFramesMin, count);
                    if (count > max) atomic_store(&context->callbackFramesMax, count);
                    *isSilence = NO;
                }
            } else { *isSilence = YES; }
            atomic_fetch_sub(&context->active, 1);
            return noErr;
        }];
    [self.engine attachNode:self.source];
    [self.engine connect:self.source to:self.engine.mainMixerNode format:format];
    [self.engine prepare];
    if (![self.engine startAndReturnError:&error]) { [self fail:[NSString stringWithFormat:@"engine: %@", error]]; return; }
    NSLog(@"NUX_AUDIO_IOS_START model=%@ system=%@ rate=%u ioDuration=%.9f outputVolume=%.3f route=%@", UIDevice.currentDevice.model,
          UIDevice.currentDevice.systemVersion, rate, session.IOBufferDuration, session.outputVolume, session.currentRoute.outputs);
    self.phase = 0;
    self.label.text = @"Nuxie audio validation\nPhase 0: generated tone + WAV asset";
    [self scheduleNext];
}
- (void)scheduleNext {
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, (int64_t)(1.5 * NSEC_PER_SEC)), dispatch_get_main_queue(), ^{
        if (!self.context || !self.context->probe) return;
        char *report = nux_audio_probe_report_json(self.context->probe);
        if (report) { NSLog(@"NUX_AUDIO_IOS_PHASE %u %s", self.phase, report); nux_audio_probe_free_json(report); }
        if (self.phase == 3) { [self finish]; return; }
        self.phase++;
        nux_audio_probe_set_phase(self.context->probe, self.phase);
        self.label.text = [NSString stringWithFormat:@"Nuxie audio validation\nPhase %u of 3", self.phase];
        [self scheduleNext];
    });
}
- (void)finish {
    AudioContext *context = self.context;
    if (context) atomic_store(&context->enabled, false);
    [self.engine stop];
    if (self.source) { [self.engine disconnectNodeOutput:self.source]; [self.engine detachNode:self.source]; }
    self.source = nil;
    self.engine = nil;
    // The source block retains the context owner until the last callback ends;
    // disabled callbacks never dereference the Rust probe.
    if (context && atomic_load(&context->active) != 0) {
        dispatch_after(dispatch_time(DISPATCH_TIME_NOW, NSEC_PER_MSEC * 10), dispatch_get_main_queue(), ^{ [self finish]; });
        return;
    }
    if (context && context->probe) {
        char *report = nux_audio_probe_report_json(context->probe);
        uint64_t calls = atomic_load(&context->callbacks), invalid = atomic_load(&context->invalidSamples);
        uint64_t wrong = atomic_load(&context->wrongLayouts);
        NSData *reportData = report ? [[NSString stringWithUTF8String:report] dataUsingEncoding:NSUTF8StringEncoding] : nil;
        id parsed = reportData ? [NSJSONSerialization JSONObjectWithData:reportData options:0 error:nil] : nil;
        BOOL probeOK = [parsed isKindOfClass:NSDictionary.class] && [parsed[@"pass"] isEqual:@YES];
        BOOL hostOK = self.phase == 3 && calls > 10 && invalid == 0 && wrong == 0 && probeOK;
        NSString *line = [NSString stringWithFormat:@"NUX_AUDIO_IOS_RESULT host_ok=%d callbacks=%llu invalid=%llu wrong_layout=%llu min_frames=%llu max_frames=%llu report=%s",
                          hostOK, calls, invalid, wrong, atomic_load(&context->callbackFramesMin), atomic_load(&context->callbackFramesMax), report ?: "null"];
        fprintf(stdout, "%s\n", line.UTF8String);
        fflush(stdout);
        NSString *path = [NSSearchPathForDirectoriesInDomains(NSDocumentDirectory, NSUserDomainMask, YES).firstObject stringByAppendingPathComponent:@"audio-validation.txt"];
        [line writeToFile:path atomically:YES encoding:NSUTF8StringEncoding error:nil];
        if (report) nux_audio_probe_free_json(report);
        nux_audio_probe_destroy(context->probe);
        context->probe = NULL;
        self.label.text = hostOK ? @"Audio validation complete\nDevice callbacks, parameter changes and teardown exercised.\nSee captured report for PCM assertions." : @"Audio validation failed\nSee captured report.";
    }
    self.context = NULL;
    self.contextOwner = nil;
    AVAudioSession *session = AVAudioSession.sharedInstance;
    [session setActive:NO withOptions:AVAudioSessionSetActiveOptionNotifyOthersOnDeactivation error:nil];
    if (self.oldCategory) [session setCategory:self.oldCategory mode:self.oldMode options:self.oldOptions error:nil];
    UIApplication.sharedApplication.idleTimerDisabled = NO;
}
@end

int main(int argc, char **argv) {
    @autoreleasepool { return UIApplicationMain(argc, argv, nil, NSStringFromClass(AudioValidationDelegate.class)); }
}
