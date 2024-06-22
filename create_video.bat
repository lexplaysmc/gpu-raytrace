@echo off
ffmpeg -framerate 30 -i frames\img%%d.png -c:v libx264 -pix_fmt yuv420p video.mp4