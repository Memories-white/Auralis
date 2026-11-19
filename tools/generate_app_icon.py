"""Generate Auralis' original vector mark and launcher compatibility PNGs.

Requires Pillow only. SVG and Android vectors share the same path definitions;
PNG rendering samples those curves at 4x resolution before downsampling.
"""
from pathlib import Path
import re
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
RES = ROOT / "android/app/src/main/res"
ASSETS = ROOT / "docs/assets"
ARCH = "M34,74 L48,38 C50.5,31.2 57.5,31.2 60,38 L74,74"
PLAY = "M49,50 C49,48.6 50.6,47.8 51.8,48.6 L60,55.1 C61.2,55.9 61.2,57.7 60,58.5 L51.8,65 C50.6,65.8 49,65 49,63.6 Z"
START = (77, 161, 255)
END = (32, 104, 232)


def write(path, content):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content + "\n", encoding="utf-8")


def sample(path):
    tokens = iter(re.findall(r"[MLCZ]|-?\d+(?:\.\d+)?", path))
    points = []
    for command in tokens:
        if command in ("M", "L"):
            points.append((float(next(tokens)), float(next(tokens))))
        elif command == "C":
            start = points[-1]
            controls = [(float(next(tokens)), float(next(tokens))) for _ in range(3)]
            for step in range(1, 33):
                t = step / 32
                points.append(tuple((1-t)**3*start[j] + 3*(1-t)**2*t*controls[0][j] + 3*(1-t)*t*t*controls[1][j] + t**3*controls[2][j] for j in (0, 1)))
        elif command == "Z":
            points.append(points[0])
    return points


def render(size, round_icon=False):
    n = size * 4
    pixels = bytearray()
    for y in range(n):
        for x in range(n):
            t = (x+y)/(2*(n-1))
            pixels.extend(round(a+(b-a)*t) for a,b in zip(START, END))
    image = Image.frombytes("RGB", (n,n), bytes(pixels)).convert("RGBA")
    mask = Image.new("L", (n,n))
    draw = ImageDraw.Draw(mask)
    if round_icon:
        draw.ellipse((0,0,n-1,n-1), fill=255)
    else:
        draw.rounded_rectangle((0,0,n-1,n-1), radius=n*24/108, fill=255)
    image.putalpha(mask)
    draw = ImageDraw.Draw(image)
    factor = n/108
    # Legacy icons retain a little more mark area because no adaptive viewport
    # magnification is applied by the launcher.
    def transform(point):
        return tuple((54+(v-54)*1.25)*factor for v in point)
    arch = [transform(p) for p in sample(ARCH)]
    width = round(8.4*factor*1.25)
    draw.line(arch, fill="white", width=width, joint="curve")
    for x,y in arch:
        r=width/2
        draw.ellipse((x-r,y-r,x+r,y+r),fill="white")
    draw.polygon([transform(p) for p in sample(PLAY)],fill="white")
    return image.resize((size,size),Image.Resampling.LANCZOS)


foreground = f'''<vector xmlns:android="http://schemas.android.com/apk/res/android"
    android:width="108dp" android:height="108dp" android:viewportWidth="108" android:viewportHeight="108">
    <path android:pathData="{ARCH}" android:fillColor="@android:color/transparent"
        android:strokeColor="#FFFFFFFF" android:strokeWidth="8.4" android:strokeLineCap="round" android:strokeLineJoin="round" />
    <path android:pathData="{PLAY}" android:fillColor="#FFFFFFFF" />
</vector>'''
write(RES / "drawable/ic_launcher_foreground.xml", foreground)
stat = foreground.replace('width="108dp"', 'width="24dp"').replace('height="108dp"', 'height="24dp"').replace('viewportWidth="108"', 'viewportWidth="72"').replace('viewportHeight="108"', 'viewportHeight="72"')
stat = stat.replace('    <path', '    <group android:translateX="-18" android:translateY="-18">\n    <path', 1).replace('</vector>', '    </group>\n</vector>')
write(RES / "drawable/ic_stat_auralis.xml", stat)
write(RES / "drawable/ic_launcher_background.xml", '''<vector xmlns:android="http://schemas.android.com/apk/res/android" xmlns:aapt="http://schemas.android.com/aapt"
    android:width="108dp" android:height="108dp" android:viewportWidth="108" android:viewportHeight="108">
    <path android:pathData="M0,0H108V108H0Z">
        <aapt:attr name="android:fillColor">
            <gradient android:type="linear" android:startX="0" android:startY="0" android:endX="108" android:endY="108" android:startColor="#4DA1FF" android:endColor="#2068E8" />
        </aapt:attr>
    </path>
</vector>''')
for api in (26,33):
    mono = '\n    <monochrome android:drawable="@drawable/ic_launcher_foreground" />' if api>=33 else ''
    adaptive = f'''<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@drawable/ic_launcher_background" />
    <foreground android:drawable="@drawable/ic_launcher_foreground" />{mono}
</adaptive-icon>'''
    for name in ("ic_launcher","ic_launcher_round"):
        write(RES / f"mipmap-anydpi-v{api}/{name}.xml",adaptive)
for density,size in (("mdpi",48),("hdpi",72),("xhdpi",96),("xxhdpi",144),("xxxhdpi",192)):
    folder=RES/f"mipmap-{density}"
    folder.mkdir(parents=True,exist_ok=True)
    render(size).save(folder/"ic_launcher.png",optimize=True)
    render(size,True).save(folder/"ic_launcher_round.png",optimize=True)
ASSETS.mkdir(parents=True,exist_ok=True)
render(512).save(ASSETS/"app-icon.png",optimize=True)
write(ASSETS/"app-icon.svg",f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 108 108" role="img" aria-labelledby="title">
    <title id="title">Auralis app icon</title>
    <defs><linearGradient id="bg" x2="1" y2="1"><stop stop-color="#4DA1FF"/><stop offset="1" stop-color="#2068E8"/></linearGradient></defs>
    <rect width="108" height="108" rx="24" fill="url(#bg)"/>
    <g transform="translate(54 54) scale(1.25) translate(-54 -54)">
        <path d="{ARCH}" fill="none" stroke="white" stroke-width="8.4" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="{PLAY}" fill="white"/>
    </g>
</svg>''')
print("Generated Android adaptive, monochrome, 10 density PNGs and SVG/PNG preview.")
