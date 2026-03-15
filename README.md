# shader_lab

A small personal playground for learning `wgpu` and `WGSL`.

The goal of this project is to make shader experiments quick to run and easy to iterate on. It currently includes:

- shader hot reload
- a simple multi-pass `noise_blur` example: `noise -> blur_h -> blur_v -> composite`
- fullscreen-pass rendering for post-processing style experiments
- a shader folder layout split into shared code and scene-specific shader sets

## Run

```bash
cargo run
```

## Controls

- `Left/Right Arrows`: cycle through available shader scenes
- `/`: enter search mode (type to filter, `Enter` to switch, `Esc` to cancel)
- `C`: toggle split comparison
- Left mouse drag: move the split line
- `Esc`: quit

## Notes

This is intentionally a small practice project, not a general-purpose renderer or engine. The structure stays explicit on purpose so shader passes are easy to read, modify, and replace.

Right now the project is mainly suited for:

- procedural shaders
- post-processing effects
- multi-pass experiments
- mouse/time-driven visual studies

It should also be a reasonable base for future experiments such as bloom, Sobel, feedback effects, SDF UI, or simple 3D post-process workflows.
