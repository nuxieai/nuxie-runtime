package ai.nuxie.runtime;

import android.content.Context;
import android.graphics.ImageFormat;
import android.graphics.Rect;
import android.hardware.HardwareBuffer;
import android.media.AudioManager;
import android.media.Image;
import android.media.ImageReader;
import android.media.MediaExtractor;
import android.media.MediaFormat;
import android.media.MediaPlayer;
import android.media.MediaTimestamp;
import android.media.PlaybackParams;
import android.os.Handler;
import android.os.HandlerThread;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicLong;

/**
 * Decoder/audio owner only: no View, no overlay, and no GL. MediaPlayer
 * decodes into an ImageReader, and each frame goes to the runtime's Vulkan
 * factory in its hardware buffer, which the renderer imports and converts on
 * the GPU; no pixels pass through the CPU. Requires Android 10 (API 29).
 * Bundle this class with the host; all media mutations use one worker.
 */
public final class VideoPlayer {
  /** A decoded frame in the decoder's hardware buffer. Close it once imported. */
  public static final class Frame {
    public final long generation;
    public final double seconds;
    public final HardwareBuffer buffer;
    /** The picture's edges in buffer pixels; decoders may pad the buffer. */
    public final int cropLeft, cropTop, cropRight, cropBottom;
    /** Clockwise rotation from buffer to display: 0, 90, 180 or 270. */
    public final int rotationDegrees;
    /**
     * Displayed width and height after the rotation, as MediaPlayer reports
     * them. Video with non-square pixels displays wider or taller than its
     * crop.
     */
    public final int displayWidth, displayHeight;
    /** Y'CbCr matrix: 1 BT.601, 2 BT.709, 3 BT.2020. */
    public final int colorMatrix;
    /** 1 limited range, 2 full range. */
    public final int colorRange;
    private final Image image;
    Frame(long generation, double seconds, Image image, int rotationDegrees,
          int displayWidth, int displayHeight, int colorMatrix, int colorRange) {
      this.generation = generation;
      this.seconds = seconds;
      this.image = image;
      this.buffer = image.getHardwareBuffer();
      Rect crop = image.getCropRect();
      this.cropLeft = crop.left;
      this.cropTop = crop.top;
      this.cropRight = crop.right;
      this.cropBottom = crop.bottom;
      this.rotationDegrees = rotationDegrees;
      this.displayWidth = displayWidth;
      this.displayHeight = displayHeight;
      this.colorMatrix = colorMatrix;
      this.colorRange = colorRange;
    }
    /** Returns the buffer to the decoder. */
    public void close() {
      buffer.close();
      image.close();
    }
  }
  public static final class Clock {
    public final long generation;
    public final double seconds, rate;
    public final boolean playing;
    Clock(long generation, double seconds, double rate, boolean playing) {
      this.generation = generation;
      this.seconds = seconds;
      this.rate = rate;
      this.playing = playing;
    }
  }
  private static final class ClockSnapshot {
    final long epoch, generation, captured, anchor;
    final double seconds, rate;
    ClockSnapshot(long epoch, long generation, long captured, long anchor,
                  double seconds, double rate) {
      this.epoch = epoch;
      this.generation = generation;
      this.captured = captured;
      this.anchor = anchor;
      this.seconds = seconds;
      this.rate = rate;
    }
  }
  private final AtomicLong clockEpoch = new AtomicLong();
  private final AtomicBoolean clockRefreshPending = new AtomicBoolean();
  private volatile ClockSnapshot clockSnapshot;
  private void invalidateClock() {
    clockEpoch.incrementAndGet();
    clockSnapshot = null;
  }
  @SuppressWarnings("deprecation")
  private void refreshClock(long epoch) {
    try {
      if (closed || !ready || seeking || failure != null ||
          clockEpoch.get() != epoch)
        return;
      MediaTimestamp timestamp = player.getTimestamp();
      if (timestamp == null || timestamp.getAnchorMediaTimeUs() < 0)
        return;
      long anchor = android.os.Build.VERSION.SDK_INT >= 29
                        ? timestamp.getAnchorSystemNanoTime()
                        : timestamp.getAnchorSytemNanoTime();
      double rate = timestamp.getMediaClockRate();
      if ((Double.isNaN(rate) || Double.isInfinite(rate)) || rate < 0)
        return;
      clockSnapshot = new ClockSnapshot(
          epoch, generation, System.nanoTime(), anchor,
          timestamp.getAnchorMediaTimeUs() / 1_000_000.0, rate);
    } catch (Exception e) {
      fail("media clock: " + e);
    } finally {
      clockRefreshPending.set(false);
    }
  }
  // All MediaPlayer calls stay on its owner thread. At most one refresh is
  // queued; stale observations are unavailable rather than extrapolated
  // forever.
  public Clock clock() {
    if (closed || failure != null)
      return null;
    long epoch = clockEpoch.get();
    if (clockRefreshPending.compareAndSet(false, true) &&
        !handler.post(() -> refreshClock(epoch)))
      clockRefreshPending.set(false);
    ClockSnapshot sample = clockSnapshot;
    long now = System.nanoTime();
    if (sample == null || sample.epoch != epoch || now - sample.captured < 0 ||
        now - sample.captured > 100_000_000L)
      return null;
    double seconds =
        sample.seconds + (now - sample.anchor) / 1_000_000_000.0 * sample.rate;
    if ((Double.isNaN(seconds) || Double.isInfinite(seconds)) || seconds < 0)
      return null;
    return new Clock(sample.generation, Math.min(seconds, duration),
                     sample.rate, playing && !ended && sample.rate > 0);
  }
  private final HandlerThread thread = new HandlerThread("NuxieVideo");
  private final Handler handler;
  private final int maxFrameBytes;
  private volatile boolean ready, playing, ended, closed;
  private volatile String failure;
  private volatile double duration;
  private Frame latest;
  private MediaPlayer player;
  private ImageReader reader;
  private int rotationDegrees, displayWidth, displayHeight, colorMatrix, colorRange;
  private long generation;
  private boolean seeking, wantsPlay;
  private double queuedSeek = -1;
  private long queuedGeneration;
  private float rate = 1, volume = 0;
  private final AudioManager audio;
  private final int audioPolicy;
  private boolean ownsFocus;
  private volatile boolean interrupted;
  private final AtomicBoolean interruptionEnded = new AtomicBoolean();
  private final AtomicBoolean permanentLoss = new AtomicBoolean();
  private final AtomicBoolean playBlocked = new AtomicBoolean();
  private final AudioManager.OnAudioFocusChangeListener focusListener =
      this::focusChanged;
  private void focusChanged(int change) {
    invalidateClock();
    handler.post(() -> {
      if (closed)
        return;
      if (change == AudioManager.AUDIOFOCUS_GAIN) {
        if (interrupted) interruptionEnded.set(true);
        interrupted = false;
        player.setVolume(volume, volume);
      } else if (change == AudioManager.AUDIOFOCUS_LOSS_TRANSIENT_CAN_DUCK) {
        player.setVolume(volume * 0.2f, volume * 0.2f);
      } else {
        if (ready)
          player.pause();
        playing = false;
        if (change == AudioManager.AUDIOFOCUS_LOSS) {
          permanentLoss.set(true);
          interrupted = false;
          ownsFocus = false;
          wantsPlay = false;
        } else
          interrupted = true;
      }
    });
  }
  public VideoPlayer(Context application, String source, long generation,
                     int maxFrameBytes, int audioPolicy) {
    this.audio =
        (AudioManager)application.getApplicationContext().getSystemService(
            Context.AUDIO_SERVICE);
    this.audioPolicy = audioPolicy;
    this.generation = generation;
    this.maxFrameBytes = maxFrameBytes;
    thread.start();
    handler = new Handler(thread.getLooper());
    handler.post(() -> open(source));
  }
  private void open(String source) {
    try {
      int[] size = readTrack(source);
      checkBudget(size[0], size[1]);
      // Four images: one the runtime is importing, the latest, one being
      // decoded and one spare. PRIVATE images take the decoder's own size.
      reader = ImageReader.newInstance(size[0], size[1], ImageFormat.PRIVATE, 4,
                                       HardwareBuffer.USAGE_GPU_SAMPLED_IMAGE);
      reader.setOnImageAvailableListener(r -> onFrame(), handler);
      player = new MediaPlayer();
      player.setVolume(0, 0);
      player.setSurface(reader.getSurface());
      player.setOnVideoSizeChangedListener((p, w, h) -> {
        try {
          checkBudget(w, h);
          displayWidth = w;
          displayHeight = h;
        } catch (Exception e) {
          fail(e.toString());
        }
      });
      player.setOnPreparedListener(p -> {
        duration = p.getDuration() / 1000.0;
        ready = true;
        if (queuedSeek >= 0) {
          double target = queuedSeek;
          long token = queuedGeneration;
          queuedSeek = -1;
          beginSeek(target, token);
        }
      });
      player.setOnCompletionListener(p -> {
        invalidateClock();
        playing = false;
        ended = true;
      });
      player.setOnErrorListener((p, what, extra) -> {
        fail("MediaPlayer " + what + ":" + extra);
        return true;
      });
      player.setOnSeekCompleteListener(p -> {
        if (queuedSeek >= 0) {
          double target = queuedSeek;
          long token = queuedGeneration;
          queuedSeek = -1;
          beginSeek(target, token);
        } else {
          seeking = false;
          if (wantsPlay && !interrupted && acquireFocus()) {
            try {
              // Rate commands received while preparing/seeking are deferred.
              // Apply the retained rate before any resumed media is played.
              p.setPlaybackParams(new PlaybackParams().setSpeed(rate));
              p.start();
              playing = true;
            } catch (Exception e) {
              fail("resume after seek: " + e);
            }
          }
        }
      });
      player.setDataSource(source);
      player.prepareAsync();
    } catch (Exception e) {
      fail(e.toString());
    }
  }
  /**
   * Width, height, rotation and color from the file's video track. The color
   * is what the decoder tags its output with: the stream's own description,
   * or Android's defaults for streams without one (limited range; BT.2020
   * from 4K, BT.601 up to 720x576, BT.709 between). The Vulkan conversion
   * needs it because drivers' own suggestions are not reliable.
   */
  private int[] readTrack(String source) throws java.io.IOException {
    MediaExtractor extractor = new MediaExtractor();
    try {
      if (source.startsWith("http://") || source.startsWith("https://"))
        extractor.setDataSource(source, new java.util.HashMap<String, String>());
      else
        extractor.setDataSource(source);
      for (int track = 0; track < extractor.getTrackCount(); track++) {
        MediaFormat format = extractor.getTrackFormat(track);
        String mime = format.getString(MediaFormat.KEY_MIME);
        if (mime == null || !mime.startsWith("video/"))
          continue;
        int width = integer(format, MediaFormat.KEY_WIDTH, 0);
        int height = integer(format, MediaFormat.KEY_HEIGHT, 0);
        rotationDegrees = integer(format, MediaFormat.KEY_ROTATION, 0);
        // Non-square pixels widen the picture. MediaPlayer's own video size
        // replaces this estimate once it reports one.
        int sarWidth = integer(format, "sar-width", 1), sarHeight = integer(format, "sar-height", 1);
        int shown = sarWidth > 0 && sarHeight > 0 ? (int)((long)width * sarWidth / sarHeight) : width;
        boolean turned = rotationDegrees % 180 != 0;
        displayWidth = Math.max(1, turned ? height : shown);
        displayHeight = Math.max(1, turned ? shown : height);
        colorMatrix = matrix(integer(format, MediaFormat.KEY_COLOR_STANDARD, 0), width, height);
        colorRange = integer(format, MediaFormat.KEY_COLOR_RANGE, 0) ==
                             MediaFormat.COLOR_RANGE_FULL ? 2 : 1;
        return new int[] {Math.max(1, width), Math.max(1, height)};
      }
      throw new IllegalArgumentException("source has no video track");
    } finally {
      extractor.release();
    }
  }
  private static int integer(MediaFormat format, String key, int fallback) {
    return format.containsKey(key) ? format.getInteger(key) : fallback;
  }
  private static int matrix(int standard, int width, int height) {
    switch (standard) {
    case MediaFormat.COLOR_STANDARD_BT601_PAL:
    case MediaFormat.COLOR_STANDARD_BT601_NTSC:
      return 1;
    case MediaFormat.COLOR_STANDARD_BT709:
      return 2;
    case MediaFormat.COLOR_STANDARD_BT2020:
      return 3;
    default:
      if (width >= 3840 || height >= 3840 || (long)width * height >= 3840L * 1634)
        return 3;
      if ((width <= 720 && height <= 576) || (height <= 720 && width <= 576))
        return 1;
      return 2;
    }
  }
  private void checkBudget(int w, int h) {
    if (w <= 0 || h <= 0 || (long)w * h * 4 > maxFrameBytes)
      throw new IllegalArgumentException("video frame exceeds budget");
  }
  private void onFrame() {
    if (closed || failure != null)
      return;
    Image image = null;
    try {
      image = reader.acquireLatestImage();
      if (image == null || seeking || !ready)
        return;
      observeDecoderInfo();
      checkBudget(displayWidth, displayHeight);
      Frame frame = new Frame(generation, player.getCurrentPosition() / 1000.0,
                              image, rotationDegrees, displayWidth, displayHeight,
                              colorMatrix, colorRange);
      image = null;
      synchronized (this) {
        if (latest != null)
          latest.close();
        latest = frame;
      }
    } catch (Exception e) {
      fail(e.toString());
    } finally {
      if (image != null)
        image.close();
    }
  }
  public void action(int action, double value, long token) {
    invalidateClock();
    if (closed)
      return;
    handler.post(() -> {
      if (closed || failure != null)
        return;
      try {
        switch (action) {
        case 0:
          wantsPlay = true;
          if (ready && !seeking && !interrupted && acquireFocus()) {
            player.setPlaybackParams(new PlaybackParams().setSpeed(rate));
            player.start();
            playing = true;
          }
          break;
        case 1:
          wantsPlay = false;
          if (ready)
            player.pause();
          playing = false;
          // Retain the focus request during transient loss so Android can
          // deliver AUDIOFOCUS_GAIN. Runtime intent decides whether to resume.
          if (!interrupted) releaseFocus();
          break;
        case 2:
          synchronized (this) {
            if (latest != null)
              latest.close();
            latest = null;
          }
          ended = false;
          if (seeking || !ready) {
            queuedSeek = value;
            queuedGeneration = token;
            if (!ready)
              generation = token;
          } else
            beginSeek(value, token);
          break;
        case 3:
          rate = (float)value;
          if (ready && playing)
            player.setPlaybackParams(new PlaybackParams().setSpeed(rate));
          break;
        case 4:
          volume = (float)value;
          if (volume == 0)
            releaseFocus();
          if (!wantsPlay || acquireFocus())
            player.setVolume(volume, volume);
          else {
            if (ready)
              player.pause();
            playing = false;
          }
          break;
        default:
          throw new IllegalArgumentException("unknown video command");
        }
      } catch (Exception e) {
        fail(e.toString());
      }
    });
  }
  @SuppressWarnings("deprecation")
  private boolean acquireFocus() {
    if (volume == 0 || audioPolicy == 1 || ownsFocus)
      return true;
    int kind = audioPolicy == 2
                   ? AudioManager.AUDIOFOCUS_GAIN_TRANSIENT_MAY_DUCK
                   : AudioManager.AUDIOFOCUS_GAIN_TRANSIENT;
    ownsFocus = audio.requestAudioFocus(focusListener,
                                        AudioManager.STREAM_MUSIC, kind) ==
                AudioManager.AUDIOFOCUS_REQUEST_GRANTED;
    if (!ownsFocus)
      playBlocked.set(true);
    return ownsFocus;
  }
  @SuppressWarnings("deprecation")
  private void releaseFocus() {
    if (ownsFocus) {
      audio.abandonAudioFocus(focusListener);
      ownsFocus = false;
    }
  }
  public boolean interrupted() { return interrupted; }
  public boolean takeInterruptionEnded() { return interruptionEnded.getAndSet(false); }
  public boolean takePermanentLoss() { return permanentLoss.getAndSet(false); }
  public boolean takePlayBlocked() { return playBlocked.getAndSet(false); }
  private void beginSeek(double seconds, long token) {
    if ((Double.isNaN(seconds) || Double.isInfinite(seconds)) || seconds < 0)
      throw new IllegalArgumentException("invalid seek");
    seeking = true;
    generation = token;
    if (android.os.Build.VERSION.SDK_INT >= 26)
      player.seekTo((long)(seconds * 1000), MediaPlayer.SEEK_CLOSEST);
    else
      player.seekTo((int)Math.min(Integer.MAX_VALUE, seconds * 1000));
  }
  private void fail(String message) {
    invalidateClock();
    failure = message;
    playing = false;
    if (player != null) {
      try {
        player.pause();
      } catch (Exception ignored) {
      }
    }
  }
  public boolean ready() { return ready; }
  public boolean playing() { return playing; }
  public boolean ended() { return ended; }
  public String failure() { return failure; }
  public double duration() { return duration; }
  public synchronized Frame takeFrame() {
    Frame frame = latest;
    latest = null;
    return frame;
  }
  public static final class DecoderInfo {
    public final String name;
    // 0 unknown, 1 reported hardware, 2 reported software.
    public final int acceleration;
    // Advisory API value, not an admission limit or observed concurrent capacity.
    public final int advertisedMaxInstances;
    DecoderInfo(String name, int acceleration, int advertisedMaxInstances) {
      this.name = name;
      this.acceleration = acceleration;
      this.advertisedMaxInstances = advertisedMaxInstances;
    }
  }
  private volatile DecoderInfo decoderInfo;
  private long nextDecoderInfoProbe;
  public DecoderInfo decoderInfo() { return closed ? null : decoderInfo; }
  private void observeDecoderInfo() {
    if (android.os.Build.VERSION.SDK_INT < 26 || decoderInfo != null || player == null) return;
    long now = android.os.SystemClock.elapsedRealtime();
    if (now < nextDecoderInfoProbe) return;
    nextDecoderInfoProbe = now + 1000;
    try {
      android.os.PersistableBundle metrics = player.getMetrics();
      if (metrics == null) return;
      String name = metrics.getString(MediaPlayer.MetricsConstants.CODEC_VIDEO);
      String mime = metrics.getString(MediaPlayer.MetricsConstants.MIME_TYPE_VIDEO);
      if (name == null || name.isEmpty()) return;
      int acceleration = 0, instances = 0;
      for (android.media.MediaCodecInfo info :
           new android.media.MediaCodecList(android.media.MediaCodecList.ALL_CODECS).getCodecInfos()) {
        if (info.isEncoder() || !info.getName().equals(name)) continue;
        if (android.os.Build.VERSION.SDK_INT >= 29) {
          if (info.isSoftwareOnly()) acceleration = 2;
          else if (info.isHardwareAccelerated()) acceleration = 1;
        }
        if (mime != null) {
          try { instances = Math.max(0, info.getCapabilitiesForType(mime).getMaxSupportedInstances()); }
          catch (IllegalArgumentException ignored) { /* Codec identity remains useful. */ }
        }
        break;
      }
      decoderInfo = new DecoderInfo(name, acceleration, instances);
    } catch (RuntimeException ignored) {
      // Optional diagnostics must not fail otherwise valid playback. Retry boundedly.
    }
  }
  private volatile RuntimeException closeFailure;
  public void close() {
    if (Thread.currentThread() == thread)
      throw new IllegalStateException("video decoder cannot synchronously close its own worker");
    synchronized (this) {
      if (!closed) {
        closed = true;
        invalidateClock();
        if (latest != null)
          latest.close();
        latest = null;
        if (!handler.post(() -> {
          try {
            releaseFocus();
            if (player != null) {
              player.release();
              player = null;
            }
            // The runtime closes each frame as soon as it has imported it.
            if (reader != null)
              reader.close();
          } catch (RuntimeException error) {
            closeFailure = error;
          } finally {
            thread.quitSafely();
          }
        })) closeFailure = new IllegalStateException("video teardown worker unavailable");
      }
    }
    // Scene admission may reuse this decoder slot immediately after close.
    // Wait for the private worker to release MediaPlayer/ImageReader first.
    if (Thread.currentThread() != thread) {
      try {
        thread.join(2000);
      } catch (InterruptedException error) {
        Thread.currentThread().interrupt();
        throw new IllegalStateException(
            "interrupted while closing video decoder", error);
      }
      if (thread.isAlive())
        throw new IllegalStateException("video decoder teardown timed out");
    }
    if (closeFailure != null)
      throw new IllegalStateException("video decoder teardown failed", closeFailure);
  }
}
