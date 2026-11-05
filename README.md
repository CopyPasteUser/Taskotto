<div align="center">

# Taskotto

**The Linux task manager, served your way.**

A planned **lightweight, customizable process manager and system monitor** for Linux, written in Rust and designed for the [RiceBowl desktop ecosystem](https://github.com/CopyPasteUser/Ricebowl).

</div>

> **Project status: planning / repository bootstrap.** There is no working task manager, installer or release yet. The features below describe the intended scope.

## The idea

Taskotto will combine the utility of a traditional process manager with the flexibility of a riceable interface:

- **Processes:** searchable and sortable process list, tree, details and resource usage.
- **Performance:** CPU, memory, disk and network activity with clear units and update controls.
- **Actions:** deliberate, permission-aware process termination and other supported Linux actions.
- **Customization:** compact/classic and modern layouts, colors, density and accessible keyboard navigation.
- **Desktop integration:** stand-alone operation first, optional RiceBowl theme synchronization later.
- **Efficiency:** low idle CPU/RAM usage, measured on real hardware rather than advertised estimates.

No extra privileged daemon, always-on background service or desktop environment will be required just to view processes.

## Planned milestones

1. **Foundation:** Rust application, Linux process snapshot model, tests and baseline benchmarks.
2. **Useful first version:** process list, filtering, details, CPU/memory meters and safe process actions.
3. **Customizable UI:** multiple layouts, persistent preferences and theme configuration.
4. **RiceBowl integration:** opt-in theme profile hand-off, packaging and hardware acceptance.

The UI toolkit and implementation details will be selected after a small performance/compatibility prototype. Taskotto is not yet a dependency of RiceBowl.

## Related projects

- [RiceBowl](https://github.com/CopyPasteUser/Ricebowl) — the modular Linux desktop environment in development.
- [RiceField](https://github.com/CopyPasteUser/Ricefield) — the separate Rust file manager, currently a 1.0 release candidate.

Licensed under [MIT](LICENSE). Contributions and design discussions can begin before the first executable release.
