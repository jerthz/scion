
<div align="right">
  <details>
    <summary >🌐 Language</summary>
    <div>
      <div align="center">
        <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=en">English</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=zh-CN">简体中文</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=zh-TW">繁體中文</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=ja">日本語</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=ko">한국어</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=hi">हिन्दी</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=th">ไทย</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=fr">Français</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=de">Deutsch</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=es">Español</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=it">Italiano</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=ru">Русский</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=pt">Português</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=nl">Nederlands</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=pl">Polski</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=ar">العربية</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=fa">فارسی</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=tr">Türkçe</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=vi">Tiếng Việt</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=id">Bahasa Indonesia</a>
        | <a href="https://openaitx.github.io/view.html?user=jerthz&project=scion&lang=as">অসমীয়া</
      </div>
    </div>
  </details>
</div>

<img src="banner.png" alt="Scion" />

Scion is a 2D game library made in rust.

> Please note that this project is in its first milestones and is subject to change according to convenience needs and big features coming.
> 
> You can use Scion as you want, although I just made this open source to serve the community not to be a competitor to the current Rust game engine ecosystem.

## Why this project? 
Well, firstly because it's a good way to learn the different aspects of game dev under the hood. 
Then, because I wanted to create something with modest goals, focused on ease of use and a short list of principles that also serves as a guideline.

Scion is not 'inspired' by any other engine, it's inspired by the needs from real game projects. This means I don't implement feature juste because I've seen it elsewhere, but because I need it for the next project that uses Scion.

### Goals
- Strong focus on **2D** only.
- **Easy** and **Fun** to use. 
- Clean and readable source code (at least we'll try to rewrite when necessary)
- Editor-less, it's a lib and should always be

### Non goals
- Ultra/over optimized code and performances. For this, please try other engines or build your own!
- 3D

## Documentations, Tutorials?
Yes, they are planned and being worked on. If you want to see scion in action, check the examples for now.

## Why ECS?
Today, ECS is like a 'magic' word for games, game libs and game engine. But ECS is not and must not be considered as a universal answer to multiple questions.
I believe that ECS has its strength and its weaknesses. 
The choice of it here is made because the main target games of this lib at its start were to be able to make: 
- a complex city building game.
- a pkmn fan game with real time trainer progression and wild pkmn.
- granular network packets representation && dot simulation games.

## Notable dependencies
These are the dependencies this project is relying on. Thanks to these awesome communities, we are able to build this kind of tiny projects. 

- <a href="https://github.com/rust-windowing/winit" target="blank">winit</a> and <a href="https://github.com/gfx-rs/wgpu/tree/master/wgpu" target="blank">wgpu</a> (windowing and multi backend rendering)
- <a href="https://github.com/Ralith/hecs" target="blank">hecs</a> (Entity component system)
- <a href="https://github.com/termhn/ultraviolet" target="blank">ultraviolet</a> (Maths)

## Pre-requisites
To build Scion, you need to install the required packages for your system: 
### Debian, Ubuntu, PopOS...
```sh
sudo apt install gcc cmake build-essential libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev
```

### Fedora
```sh
sudo dnf install gcc cmake make automake gcc gcc-c++ kernel-devel libX11-devel libXi-devel mesa-libGL-devel alsa-lib-devel
```

### Arch, Manjaro...
```sh
sudo pacman -S gcc cmake make automake linux-headers libx11 libxi mesa alsa-lib
```

### Run an example
Then, you can run an example:
```sh
cargo run --example bomberman --release
```

## Showcase

| <img src="./examples/taquin/taquin_2.gif" alt="Taquin" style="width:300px; border: 2px solid black;"/>  | <img src="./examples/tetris/tetris.gif" alt="Tetris" style="width:300px" />             | <img src="./examples/pixel-adventures/pixel-adventures.gif" alt="Pixel-adventures" style="width:300px" /> |
|---------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------|
| <img src="./examples/bomberman/bomberman.gif" alt="Bomberman" style="width:300px" />                    | <img src="./examples/jezzball/jezzball.gif" alt="Jezzball" style="width:300px" />       | <img src="./examples/new-bark-town/new-bark-town.gif" alt="New bark town" style="width:300px" />          |
| <img src="./examples/starlight-1961/starlight.gif" alt="Starlight" style="width:300px" />               | <img src="./examples/klondike.gif" alt="Starlight" style="width:300px" /> | |

