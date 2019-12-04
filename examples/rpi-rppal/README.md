# Demo for Raspberry Pi

Running for RPI is pretty simple. This demo has user friendly CLI interface.

All you have to do is copy binary to RPi and launch it.
You'll be provided with CLI properties you should enter.

You should provide RPi pin numbers where you've connected TM16xx module.
This is how you can run 3 wire demo.
```bash
rpi-rppal --dio=20 --clk=21 --stb=16
```
For 2 wire demo just skip `--stb` argument.
