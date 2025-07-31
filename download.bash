#!/bin/bash
# remove possible previous
rm ./*.webm
rm ./*.mp4
rm ./*.avi
rm ./*.mkv
# download
while read LINE; do yt-dlp -f bv[height=144]+ba --cookies-from-browser firefox "$LINE"; done < links.txt
# convert all
for i in *.webm; do ffmpeg -i "$i" "${i%.*}.mp3" -q:a 0; done
for i in *.mp4; do ffmpeg -i "$i" "${i%.*}.mp3" -q:a 0; done
for i in *.avi; do ffmpeg -i "$i" "${i%.*}.mp3" -q:a 0; done
for i in *.mkv; do ffmpeg -i "$i" "${i%.*}.mp3" -q:a 0; done
# remove old non-mp3
rm ./*.webm
rm ./*.mp4
rm ./*.avi
rm ./*.mkv
