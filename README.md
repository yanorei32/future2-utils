# CLI Utilities for 미래2.0
<img width="450" height="357" alt="Stad20 big 0 data" src="https://github.com/user-attachments/assets/fd5aec7a-0338-479f-aad2-924beab5b891" />

## future2-bigfile-unpack

```
$ future2-bigfile-unpack -i Stad12.big
```

<img width="267" src="https://github.com/user-attachments/assets/9ffb4572-b79f-4fa4-a219-df1cde205354" />

## future2-bigfile-pack

```
$ future2-bigfile-pack -i MyStad12.big.0.data -i MyStad12.big.1.data -i MyStad12.big.2.data -o MyStad12.big
```

## future2-image-decode

```
$ future2-image-decode -i /mnt/r/Stad20.big.0.data
450x357 Rgb555_16bpc
```

In this case, the output file name is: `Stad20.big.0.data.png`

## future2-image-encode

```
$ future2-image-encode -i /mnt/r/Stad20.big.0.data.png
```

In this case, the output file name is: `Stad20.big.0.data.png.data`


## future2-s10-unpack

```
$ future2-s10-pack -i Data/S10.str
```

<img width="321" src="https://github.com/user-attachments/assets/c4874c4d-1617-41cd-bf5b-a6b123babbfd" />

## future2-s10-pack

```
$ future2-s10-pack -i 攻撃戦だ.mp3 -i ウサテイ.MP3 -i "A Turtle's Heart.mp3" -o Custom.s10.str
```

<img width="467" src="https://github.com/user-attachments/assets/b0ae1528-85c4-47bb-9b0e-c37186c1fc22" />
