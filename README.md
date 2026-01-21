# madrlib / madrctl

Inspired by my other project `mxw`. I got a new mouse specifically for gaming, the [VXE MAD R](https://www.atk.store/products/vxe-mad-r-series-wireless-mouse),
and decided I wanted to make a similar project, but by reverse engineering the mouse myself instead of taking the heavy lifting from someone else. Consider reading the associated [blog post](https://bednarczyk.xyz/blog/reverse-engineering-a-gaming-mouse/).

In theory, this should also work with any other ATK VXE MAD R device.

The goal is to have full feature parity with the [web interface](https://v3-hub.atk.store/).

This project is split into two parts, a library and a generic CLI tool that implements every aspect of said library.

## Support
- [x] DPI stages
    - [x] Set active DPI stage
    - [x] Set DPI for a given stage
    - [x] Set accent color for a given stage
    - [ ] Add/remove DPI stages
- [x] Polling rate
- [x] Sensor sampling rate
- [x] Debounce time
- [x] Sleep time
- [x] Battery (percentage, voltage, charging status)
- [ ] LOD Silent Height (liftoff distance)

*... and more ...*
