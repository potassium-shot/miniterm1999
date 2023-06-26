# miniterm 1999

Hello! *miniterm 1999* is an **open source**, **GPU based** terminal emulator for **Linux**, written in **Rust** with [wgpu](https://github.com/gfx-rs/wgpu).

*Current state of developement:*

![Current state of developement](https://raw.githubusercontent.com/potassium-shot/miniterm1999/master/screenshot.png)

At first, it was meant to imitated old school terminals, such as the kind of terminals you might be used to in Fallout 4.

*Example of a Fallout terminal:*

![Example of a Fallout terminal](https://static.wikia.nocookie.net/fallout/images/7/74/Terminal.jpg/revision/latest?cb=20170919035537)

This is done using shaders. But I quickly realised, if you can have old-school VHS like shaders, might as well allow custom shaders as well.
The renderer is optimized, rendering text from a buffer of characters, with their background and foreground colors, on the GPU. Every time characters are modified, the text is rendered on a texture, which is then rendered to the screen by a second render pipeline, responsible for applying the custom shader.
The project is currently a WIP.
