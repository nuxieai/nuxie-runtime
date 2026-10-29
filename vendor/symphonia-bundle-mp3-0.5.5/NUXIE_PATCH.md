# Nuxie patch

Crates.io `symphonia-bundle-mp3` 0.5.5, checksum
`4872dd6bb56bf5eac799e3e957aa1981086c3e613b27e0ac23b176054f7c57ed`.

Removes the unused `symphonia-metadata` manifest dependency. Decoder sources are unchanged. The demuxer adds
`MpaReader::establish_exact_frame_count`, an opt-in scan of complete audio-frame
headers that excludes Info/Xing/VBRI packets and applies existing gapless
padding/delay. This corrects inaccurate header frame counts before decoding and
seeking, corresponding to Rive runtime `aec8aea4172642f09722295e0feb28f9e07dea30`
and its miniaudio reader seek-table behavior. The scan restores the input position,
does not decode PCM, and leaves default Symphonia construction behavior unchanged.
Nuxie's audio readers and encoded-playback whole-source decoders opt into the scan.
Both upstream `AudioSource::makeReader` and `AudioEngine::play` set
`seekPointCount = AudioSource::kDecoderSeekPoints`. Source validation remains
unscanned. See `docs/symphonia-fork.md`.

The pinned miniaudio `rive_changes_9` → `rive_changes_10` delta is the single
assignment `pMP3->totalPCMFrameCount = totalPCMFrameCount` after
`ma_dr_mp3_get_mp3_and_pcm_frame_count` in `ma_dr_mp3_calculate_seek_points`
(commit `5f5977d972e0bbe5cd53063db601710df2418139`). That scan omits PCM output
and preserves the read position; the existing delay/padding correction remains
in `ma_dr_mp3_get_pcm_frame_count`. Our existing Symphonia gapless convention
remains the approved D17 adaptation.
