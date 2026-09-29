# Video timing fixture

`red-blue-audio.mp4` is generated test content: one second red, one second blue,
64×32, 30 fps H.264 baseline, plus a two-second 880 Hz AAC track. It contains no
third-party source footage. The primary oracle is red before 0.9 s and blue
after 1.1 s, including after a seek to 1.3 s.

Reproduce with FFmpeg:

```sh
ffmpeg -f lavfi -i 'color=c=red:s=64x32:r=30:d=1' \
  -f lavfi -i 'color=c=blue:s=64x32:r=30:d=1' \
  -f lavfi -i 'sine=frequency=880:sample_rate=48000:duration=2' \
  -filter_complex '[0:v][1:v]concat=n=2:v=1:a=0[v]' \
  -map '[v]' -map 2:a -c:v libx264 -profile:v baseline -pix_fmt yuv420p \
  -c:a aac -movflags +faststart red-blue-audio.mp4
```

Default automated platform proofs mute the audio track. The Apple fixture app
also supports audible playback, heard on the Bat Phone, and audible lifecycle
qualification. These checks do not measure A/V skew or establish hardware
acceleration.

`red-blue-sync.mp4` extends the final blue frame and silent audio padding to ten
seconds so multi-player startup and seek correction can finish on slower hosts.
The red/blue pixel and caption timing oracles stay unchanged. Generate it from
the first fixture:

```sh
ffmpeg -i red-blue-audio.mp4 \
  -vf 'tpad=stop_mode=clone:stop_duration=8' -af 'apad=pad_dur=8' \
  -t 10 -c:v libx264 -pix_fmt yuv420p -c:a aac \
  -movflags +faststart red-blue-sync.mp4
```

`red-blue-720p.mp4` uses the first command above with both video sizes changed
to `1280x720`. Its frame rate, colors, two-second duration, H.264 baseline, AAC
tone and timing oracle are otherwise identical. It exercises sustained native
frame copying, GPU upload, rendering and reclamation at 720p. This benchmark
fixture is not a product resolution limit.

`red-blue-endpoint.mp4` reduces the original fixture to 10fps while retaining
its audio. Its last video sample starts at 1.9s and the browser track ends at
2.0s, exceeding the ordinary 50ms seek tolerance. The browser endpoint proof
must settle using the decoder-selected frame's actual timestamp and blue pixels.

```sh
ffmpeg -i red-blue-audio.mp4 -vf fps=10 -c:v libx264 \
  -pix_fmt yuv420p -c:a copy red-blue-endpoint.mp4
```

`parity-opaque.mp4` and `parity-alpha.webm` feed the browser upload parity
proof (`tools/video-qualification/browser/parity.html`). Both are FFmpeg's
`testsrc2` pattern at 64×32 and 10 fps for one second: asymmetric in both axes,
so a flipped or mirrored upload cannot match. The WebM is VP9 with an alpha
plane that ramps from 16 on the left to 254 on the right, so premultiplication
errors show in every column.

```sh
ffmpeg -f lavfi -i 'testsrc2=s=64x32:r=10:d=1' -c:v libx264 -profile:v baseline \
  -pix_fmt yuv420p -movflags +faststart parity-opaque.mp4
ffmpeg -f lavfi -i 'testsrc2=s=64x32:r=10:d=1' \
  -f lavfi -i "color=c=black:s=64x32:r=10:d=1,format=gray,geq=lum='16+X*239/63'" \
  -filter_complex '[0:v]format=yuv420p[c];[c][1:v]alphamerge,format=yuva420p[v]' \
  -map '[v]' -c:v libvpx-vp9 -pix_fmt yuva420p -auto-alt-ref 0 -b:v 0 -crf 20 \
  parity-alpha.webm
```
