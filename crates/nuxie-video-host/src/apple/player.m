#import <Foundation/Foundation.h>
#import <AVFoundation/AVFoundation.h>
#import <CoreVideo/CoreVideo.h>
#import <QuartzCore/QuartzCore.h>
#import <TargetConditionals.h>
#include <stdint.h>
#include <math.h>
#include <string.h>

// One retained player per scene occurrence; no views/layers and no global
// player. The caller uses the main thread and retains the handle until close.
@interface NuxVideoPlayer : NSObject
@property AVPlayer *player;
@property AVPlayerItem *item;
@property AVPlayerItemVideoOutput *output;
@property uint64_t generation;
@property BOOL seeking;
@property BOOL frameRequestedAfterSeek;
@property BOOL opened;
@property BOOL ended;
@property BOOL playing;
@property BOOL failed;
@property float requestedRate;
@property CVPixelBufferRef pendingFrame;
@property double frameTime;
@property id endObserver;
@property NSArray *audioObservers;
@property BOOL interrupted;
@property BOOL pauseRequested;
@property BOOL interruptionEnded;
@property BOOL playBlocked;
@property BOOL wantsPlay;
@property BOOL managesAudioSession;
@property uint32_t audioPolicy;
- (BOOL)prepareAudio;
- (void)releaseAudio;
@end

// Only opt-in runtime-managed players participate. An embedding app with its
// own audio engine keeps ownership and uses host-managed mode instead.
#if TARGET_OS_IOS || TARGET_OS_TV || TARGET_OS_VISION
static NSHashTable<NuxVideoPlayer *> *audioPlayers;
static NSString *savedCategory;
static NSString *savedMode;
static AVAudioSessionCategoryOptions savedOptions;
static AVAudioSessionCategoryOptions installedOptions;
static BOOL ownsAudioSession;
static BOOL updateAudioSession(void) {
    AVAudioSession *session = [AVAudioSession sharedInstance];
    if (audioPlayers.count == 0) {
        if (!ownsAudioSession) return YES;
        // A host may take ownership while video is active. Never overwrite its
        // later configuration or deactivate its audio in that case.
        BOOL unchanged = [session.category isEqual:AVAudioSessionCategoryPlayback]
            && [session.mode isEqual:AVAudioSessionModeDefault]
            && session.categoryOptions == installedOptions;
        ownsAudioSession = NO;
        if (!unchanged) return YES;
        NSError *error = nil;
        BOOL ok = [session setActive:NO withOptions:AVAudioSessionSetActiveOptionNotifyOthersOnDeactivation error:&error];
        if (ok) ok = [session setCategory:savedCategory mode:savedMode options:savedOptions error:&error];
        return ok;
    }
    AVAudioSessionCategoryOptions options = AVAudioSessionCategoryOptionMixWithOthers;
    for (NuxVideoPlayer *player in audioPlayers) {
        if (player.audioPolicy == 0 || player.audioPolicy == 3) { options = 0; break; }
        if (player.audioPolicy == 2) options |= AVAudioSessionCategoryOptionDuckOthers;
    }
    if (!ownsAudioSession) {
        savedCategory = session.category; savedMode = session.mode;
        savedOptions = session.categoryOptions;
    }
    NSError *error = nil;
    if (![session setCategory:AVAudioSessionCategoryPlayback mode:AVAudioSessionModeDefault options:options error:&error]) return NO;
    installedOptions = options;
    ownsAudioSession = YES;
    return [session setActive:YES error:&error];
}
#endif
@implementation NuxVideoPlayer
- (BOOL)prepareAudio {
#if TARGET_OS_IOS || TARGET_OS_TV || TARGET_OS_VISION
    if (self.managesAudioSession && self.player.volume > 0 && self.wantsPlay && !self.interrupted) {
        if (!audioPlayers) audioPlayers = [NSHashTable weakObjectsHashTable];
        [audioPlayers addObject:self];
        if (!updateAudioSession()) { [audioPlayers removeObject:self]; updateAudioSession(); return NO; }
    }
#endif
    return YES;
}
- (void)releaseAudio {
#if TARGET_OS_IOS || TARGET_OS_TV || TARGET_OS_VISION
    if ([audioPlayers containsObject:self]) { [audioPlayers removeObject:self]; updateAudioSession(); }
#endif
}
- (void)dealloc {
    if (_endObserver) [[NSNotificationCenter defaultCenter] removeObserver:_endObserver];
    for (id observer in _audioObservers) [[NSNotificationCenter defaultCenter] removeObserver:observer];
    [_player pause];
    [self releaseAudio];
    [_player replaceCurrentItemWithPlayerItem:nil];
    if (_pendingFrame) CVPixelBufferRelease(_pendingFrame);
}
@end

bool nux_video_apple_is_main_thread(void) { return [NSThread isMainThread]; }
void *nux_video_apple_open(const char *source, uint64_t generation) {
    @autoreleasepool {
        NSString *text = [NSString stringWithUTF8String:source];
        NSURL *url = [text hasPrefix:@"/"] ? [NSURL fileURLWithPath:text] : [NSURL URLWithString:text];
        if (!url) return NULL;
        NuxVideoPlayer *p = [NuxVideoPlayer new];
        p.generation = generation;
        p.requestedRate = 1.0;
        p.item = [AVPlayerItem playerItemWithURL:url];
        p.output = [[AVPlayerItemVideoOutput alloc] initWithPixelBufferAttributes:@{
            (NSString *)kCVPixelBufferPixelFormatTypeKey: @(kCVPixelFormatType_32BGRA),
            (NSString *)kCVPixelBufferMetalCompatibilityKey: @YES
        }];
        p.output.suppressesPlayerRendering = YES;
        [p.item addOutput:p.output];
        p.player = [AVPlayer playerWithPlayerItem:p.item];
        p.player.actionAtItemEnd = AVPlayerActionAtItemEndPause;
        // Runtime explicitly applies volume before issuing play.
        p.player.volume = 0.0;
        __weak NuxVideoPlayer *weak = p;
        p.endObserver = [[NSNotificationCenter defaultCenter]
            addObserverForName:AVPlayerItemDidPlayToEndTimeNotification object:p.item
            queue:[NSOperationQueue mainQueue] usingBlock:^(NSNotification *n) {
                NuxVideoPlayer *owner = weak;
                if (owner && !owner.seeking) { owner.ended = YES; owner.wantsPlay = NO; [owner releaseAudio]; }
            }];
        #if TARGET_OS_IOS || TARGET_OS_TV || TARGET_OS_VISION
        NSNotificationCenter *center = [NSNotificationCenter defaultCenter];
        id interruption = [center addObserverForName:AVAudioSessionInterruptionNotification
            object:[AVAudioSession sharedInstance] queue:[NSOperationQueue mainQueue]
            usingBlock:^(NSNotification *n) {
                NuxVideoPlayer *owner = weak;
                // Muted motion does not own the audio session. A suspension
                // interruption may never emit an end edge for silent playback.
                if (!owner || owner.player.volume == 0) return;
                NSUInteger type = [n.userInfo[AVAudioSessionInterruptionTypeKey] unsignedIntegerValue];
                if (type == AVAudioSessionInterruptionTypeBegan) {
                    owner.interrupted = YES; [owner.player pause]; owner.playing = NO;
                } else {
                    BOOL resume = ([n.userInfo[AVAudioSessionInterruptionOptionKey] unsignedIntegerValue]
                        & AVAudioSessionInterruptionOptionShouldResume) != 0;
                    if (!resume) { owner.pauseRequested = YES; owner.wantsPlay = NO; }
                    owner.interrupted = NO; owner.interruptionEnded = YES;
                }
            }];
        id route = [center addObserverForName:AVAudioSessionRouteChangeNotification
            object:[AVAudioSession sharedInstance] queue:[NSOperationQueue mainQueue]
            usingBlock:^(NSNotification *n) {
                NuxVideoPlayer *owner = weak;
                if ([n.userInfo[AVAudioSessionRouteChangeReasonKey] unsignedIntegerValue]
                        == AVAudioSessionRouteChangeReasonOldDeviceUnavailable && owner.player.volume > 0) {
                    owner.pauseRequested = YES; owner.wantsPlay = NO;
                    [owner.player pause]; owner.playing = NO; [owner releaseAudio];
                }
            }];
        p.audioObservers = @[interruption, route];
#endif
        return (__bridge_retained void *)p;
    }
}
// Call before playback. Runtime-managed mode requires the embedding app to
// delegate its process-wide audio-session ownership for the player's lifetime.
void nux_video_apple_audio_policy(void *handle, uint32_t policy, bool managed) {
    NuxVideoPlayer *p = (__bridge NuxVideoPlayer *)handle;
    p.audioPolicy = policy; p.managesAudioSession = managed;
}
uint32_t nux_video_apple_audio_status(void *handle) {
    NuxVideoPlayer *p = (__bridge NuxVideoPlayer *)handle;
    uint32_t flags = (p.interrupted ? 1 : 0) | (p.pauseRequested ? 2 : 0) | (p.playBlocked ? 4 : 0) | (p.interruptionEnded ? 8 : 0);
    p.pauseRequested = NO; p.playBlocked = NO; p.interruptionEnded = NO;
    return flags;
}
void nux_video_apple_close(void *handle) {
    @autoreleasepool { NuxVideoPlayer *p = (__bridge_transfer NuxVideoPlayer *)handle; [p.player pause]; [p releaseAudio]; }
}
void nux_video_apple_action(void *handle, int action, double value, uint64_t generation) {
    @autoreleasepool {
        NuxVideoPlayer *p = (__bridge NuxVideoPlayer *)handle;
        switch (action) {
            case 0:
                p.wantsPlay = YES;
                if (!p.interrupted) {
                    if ([p prepareAudio]) [p.player playImmediatelyAtRate:p.requestedRate];
                    else p.playBlocked = YES;
                }
                break;
            case 1: p.wantsPlay = NO; [p.player pause]; p.playing = NO; [p releaseAudio]; break;
            case 2: {
                p.generation = generation;
                p.seeking = YES; p.ended = NO; p.frameRequestedAfterSeek = NO;
                if (p.pendingFrame) { CVPixelBufferRelease(p.pendingFrame); p.pendingFrame = NULL; }
                __weak NuxVideoPlayer *weak = p;
                [p.player seekToTime:CMTimeMakeWithSeconds(value, 60000)
                    toleranceBefore:kCMTimeZero toleranceAfter:kCMTimeZero
                    completionHandler:^(BOOL finished) {
                        dispatch_async(dispatch_get_main_queue(), ^{
                            NuxVideoPlayer *owner = weak;
                            if (owner && owner.generation == generation) {
                                owner.seeking = NO;
                                owner.frameRequestedAfterSeek = finished;
                                if (!finished) owner.failed = YES;
                            }
                        });
                    }];
                break;
            }
            case 3: p.requestedRate = value; if (p.player.rate != 0.0) p.player.rate = value; break;
            case 4:
                p.player.volume = value;
                p.player.muted = value == 0;
                if (value == 0) {
                    [p releaseAudio];
                    if (p.interrupted) { p.interrupted = NO; p.interruptionEnded = YES; }
                }
                else if (![p prepareAudio]) { [p.player pause]; p.playing = NO; p.playBlocked = YES; }
                break;
        }
    }
}
// Returns one observation at a time. 1=ready,2=playing,3=ended,4=frame,
// 5=frame selected by a completed seek, -1=failure,0=no event. Decode surfaces are retained until copied/replaced.
int nux_video_apple_poll(void *handle, uint64_t *generation, double *time,
                         uint32_t *width, uint32_t *height) {
    @autoreleasepool {
        NuxVideoPlayer *p = (__bridge NuxVideoPlayer *)handle;
        *generation = p.generation;
        if (p.failed || p.item.status == AVPlayerItemStatusFailed) return -1;
        if (p.item.status != AVPlayerItemStatusReadyToPlay) return 0;
        if (!p.opened) {
            double duration = CMTimeGetSeconds(p.item.duration);
            if (!isfinite(duration) || duration < 0) return 0;
            p.opened = YES; *time = duration; return 1;
        }
        if (p.seeking) return 0;
        if (p.ended) { p.ended = NO; return 3; }
        if (!p.playing && p.player.timeControlStatus == AVPlayerTimeControlStatusPlaying) {
            p.playing = YES; return 2;
        }
        CMTime position = p.player.currentTime;
        // A completed seek can select the same decoded image as the previous
        // generation. AVPlayerItemVideoOutput then reports no *new* buffer, but
        // copyPixelBufferForItemTime still returns the selected frame and its
        // actual timestamp. Request it once for this seek owner, without keeping
        // another cached surface or treating the requested time as decoded PTS.
        if (!p.frameRequestedAfterSeek && ![p.output hasNewPixelBufferForItemTime:position]) return 0;
        CMTime actual;
        CVPixelBufferRef frame = [p.output copyPixelBufferForItemTime:position itemTimeForDisplay:&actual];
        if (!frame) return 0;
        BOOL selectedAfterSeek = p.frameRequestedAfterSeek;
        p.frameRequestedAfterSeek = NO;
        if (p.pendingFrame) CVPixelBufferRelease(p.pendingFrame);
        p.pendingFrame = frame; p.frameTime = CMTimeGetSeconds(actual);
        *width = (uint32_t)CVPixelBufferGetWidth(frame);
        *height = (uint32_t)CVPixelBufferGetHeight(frame);
        *time = p.frameTime;
        return selectedAfterSeek ? 5 : 4;
    }
}
bool nux_video_apple_copy_rgba(void *handle, uint8_t *out, size_t capacity) {
    NuxVideoPlayer *p = (__bridge NuxVideoPlayer *)handle;
    CVPixelBufferRef frame = p.pendingFrame;
    if (!frame) return false;
    size_t w = CVPixelBufferGetWidth(frame), h = CVPixelBufferGetHeight(frame);
    if (h == 0 || w > SIZE_MAX / h / 4 || capacity < w*h*4) return false;
    if (CVPixelBufferLockBaseAddress(frame, kCVPixelBufferLock_ReadOnly) != kCVReturnSuccess) return false;
    const uint8_t *data = CVPixelBufferGetBaseAddress(frame);
    size_t stride = CVPixelBufferGetBytesPerRow(frame);
    for (size_t y = 0; y < h; y++) for (size_t x = 0; x < w; x++) {
        const uint8_t *b = data + y*stride + x*4;
        uint8_t *r = out + (y*w+x)*4;
        r[0]=b[2]; r[1]=b[1]; r[2]=b[0]; r[3]=b[3];
    }
    CVPixelBufferUnlockBaseAddress(frame, kCVPixelBufferLock_ReadOnly);
    CVPixelBufferRelease(frame); p.pendingFrame = NULL;
    return true;
}
bool nux_video_apple_clock(void *handle, uint64_t *generation, double *seconds, double *rate, bool *playing) {
    NuxVideoPlayer *p = (__bridge NuxVideoPlayer *)handle;
    if (p.failed || p.seeking || p.item.status != AVPlayerItemStatusReadyToPlay) return false;
    double position = CMTimeGetSeconds(p.player.currentTime);
    if (!isfinite(position) || position < 0) return false;
    *generation = p.generation;
    *seconds = position;
    *playing = p.player.timeControlStatus == AVPlayerTimeControlStatusPlaying;
    *rate = *playing ? p.player.rate : 0;
    return true;
}
// For headless native qualification only. Application hosts own their runloop.
void nux_video_apple_pump(double seconds) {
    @autoreleasepool {
        [[NSRunLoop mainRunLoop] runUntilDate:[NSDate dateWithTimeIntervalSinceNow:seconds]];
    }
}
