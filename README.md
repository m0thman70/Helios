```
     ___   __  __      
   /   | / /_/ /_____ 
  / /| |/ __/ __/ __ \
 / ___ / /_/ /_/ /_/ /
/_/  |_\__/\__/\____/ 

```

### HELP WANTED!:
i need someone to create a logo for atto

this is atto, a new code editor similar to nano,vim and emacs that came to existance with alot of AI and ALOT of Bullsh** but here we are, it uses crossterm and tui libraries


## Bugs I am aware of:

none


### Deps:

  - rust 
    
> [!NOTE]  
> If you do not have rust currently please install the rust toolchain. It can be found at crates.io @ the install cargo button.

### Notes:

currently its in very early stages, this is version 0.0.3 so be sure to know that there is bugs 

### Binds:

`ESC` -> to see binds

`CTRL + Q` -> exit

`arrows` -> move cursor

`CTRL + W` -> save file

`CTRL + R` -> reload file

### Next steps:

So after some people used it, here is a list of a few things that people seemingly want to see:

- syntax highlighting -> possibly tree sitter
- commands -> no idea how
- a few people adsked for modes but thats gonna be sidelined

### Install:

```curl -fsSL https://raw.githubusercontent.com/m0thman70/Atto/main/install.sh | sh``` 


> [!WARNING]
> Does not update currently installed version.


### Compile:

`git clone https://github.com/m0thman70/Atto`

`cd Atto`

`cargo build --release`

`cd target/release`

`chmod +x atto`

`./atto`

and boom you got atto compiled

also atto does not have the capability to create files, please use `touch`  

if you wanna insall, instead of `cargo build` do `cargo install --path <path>`
