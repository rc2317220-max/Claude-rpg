"""Original field-journal panorama and compass. Coordinates are source pixels."""
import math
import random


def polygon(canvas, points, color):
    """Fill a closed path at pixel centers, preserving gaps between spans."""
    edges = list(zip(points, points[1:] + points[:1]))
    top = max(0, math.floor(min(y for _, y in points)))
    bottom = min(canvas.h, math.ceil(max(y for _, y in points)))
    for row in range(top, bottom):
        y = row + 0.5
        hits = sorted(x1 + (y - y1) * (x2 - x1) / (y2 - y1)
                      for (x1, y1), (x2, y2) in edges
                      if min(y1, y2) <= y < max(y1, y2))
        for left, right in zip(hits[::2], hits[1::2]):
            x0 = math.ceil(left - 0.5)
            canvas.fill(x0, row, math.ceil(right - 0.5) - x0, 1, color)


def ink(code):
    return tuple(bytes.fromhex(code)) + (255,)


def blend(start, end, weight):
    return tuple(round(a + (b - a) * weight) for a, b in zip(start, end))


def ridge(canvas, points, color):
    polygon(canvas, points + [(points[-1][0], canvas.h), (points[0][0], canvas.h)], color)


def pine(canvas, x, ground, height, color):
    top = ground - height
    canvas.fill(x, top, 1 + height // 28, height, color)
    for tip, base, width in ((0.02, 0.40, 0.10), (0.23, 0.63, 0.145), (0.40, 0.80, 0.19), (0.54, 0.93, 0.22)):
        y = top + height * base
        half = height * width
        polygon(canvas, [(x, top + height * tip), (x + half, y - 1),
                         (x + half * 0.4, y - 2), (x + 1, y),
                         (x - half * 0.65, y - 2), (x - half, y)], color)


def panorama(Canvas):
    canvas = Canvas(512, 160)
    rng = random.Random(3971)
    # Indigo at the zenith gives way to diffuse gold around the setting moon.
    night, veil, light = ink('1c353e'), ink('64938d'), ink('b8a77b')
    for y in range(canvas.h):
        sky = blend(night, veil, min(1.0, y / 112))
        for x in range(canvas.w):
            haze = max(0, 1 - math.hypot((x - 377) / 185, (y - 55) / 120))
            pixel = blend(sky, light, haze * 0.65)
            canvas.px(x, y, pixel)
    # Sparse stars and short, broken cirrus, elongated by prevailing winds.
    for _ in range(52):
        x, y = rng.randrange(149, 507), rng.randrange(3, 63)
        canvas.px(x, y, (228, 221, 176, rng.randrange(60, 143)))
    for _ in range(30):
        x, y = rng.randrange(130, 516), rng.randrange(13, 79)
        length = rng.randrange(8, 39)
        canvas.line(x, y, x + length, y, (213, 211, 177, rng.randrange(20, 45)))
    moon = ink('edcd8b')
    for y in range(-15, 16):
        half = int(math.sqrt(15**2 - y*y))
        canvas.fill(378 - half, 42 + y, 2 * half + 1, 1, moon)
    for x, y, w in ((348,45,59),(367,47,42),(373,49,21)):
        canvas.fill(x, y, w, 1, (148, 166, 147, 175))

    # Different atmospheric depths also read as distinct values in grayscale.
    ridge(canvas, [(0,83),(52,62),(74,74),(113,50),(133,67),(147,58),
                   (168,66),(204,42),(227,67),(247,50),(269,72),(282,66),
                   (301,77),(326,54),(339,63),(365,43),(382,59),(407,74),
                   (448,46),(462,54),(479,43),(512,66)], ink('75928b'))
    ridge(canvas, [(0,91),(27,79),(51,95),(108,71),(150,99),(184,83),
                   (204,66),(217,72),(247,45),(254,49),(278,22),(292,37),
                   (298,35),(323,62),(334,69),(343,65),(371,84),(394,71),
                   (429,86),(459,67),(473,72),(512,53)], ink('476871'))
    # The east faces and thin ledges catch the remaining light.
    for face in (
        [(204,66),(216,72),(237,95),(215,82)],
        [(247,45),(255,49),(271,73),(260,65),(256,69)],
        [(278,22),(292,37),(298,35),(323,62),(336,73),(317,65),(306,61),
         (296,49),(293,50),(285,37),(281,34)],
        [(343,65),(371,84),(360,82),(352,77)],
        [(394,71),(422,84),(407,83)],
        [(459,67),(474,72),(482,79),(466,73)],
    ):
        polygon(canvas, face, ink('a4aca0'))
    for face in (
        [(278,22),(270,43),(271,48),(256,65),(265,59),(276,47),(284,62),(279,46),(280,40)],
        [(298,35),(297,43),(313,62),(307,50)],
        [(247,45),(241,63),(228,78),(245,64),(251,69)],
        [(315,55),(324,72),(345,83),(332,71)],
    ):
        polygon(canvas, face, ink('607d82'))
    # A fog bank breaks the bases of the peaks before the darker tree line.
    for y in range(77, 102):
        canvas.fill(0, y, 512, 1, (157, 179, 161, int(34 * math.sin(math.pi * (y - 77) / 25))))
    ridge(canvas, [(0,107),(50,95),(70,98),(111,90),(151,101),(167,98),
                   (205,108),(236,98),(254,103),(286,87),(302,90),
                   (324,78),(345,81),(374,92),(399,88),(438,104),(468,94),
                   (488,98),(512,83)], ink('3e6465'))
    for x in range(257, 396, 3):
        ground = 93 + int((abs(327 - x) / 8)) - int(8 * math.cos((x-325)/24))
        pine(canvas, x, ground, rng.randrange(5, 12), ink('3e6465'))
    ridge(canvas, [(0,114),(34,101),(74,108),(104,101),(145,111),(180,105),
                   (208,114),(235,110),(270,117),(298,108),(325,115),
                   (359,108),(397,115),(416,108),(465,119),(512,97)], ink('527a76'))
    # River in the valley, marked with a few parallel glints.
    ridge(canvas, [(0,127),(104,116),(159,124),(216,117),(254,125),
                   (283,117),(300,119),(336,115),(377,122),(409,118),(482,110),(512,117)], ink('709083'))
    for x, y, w in ((248,128,18),(319,122,23),(344,124,11),(301,130,50),(374,124,13)):
        canvas.fill(x, y, w, 1, ink('b6b398'))

    # Broken vaulted gateway. A single concave contour leaves the arch opening
    # transparent to the actual landscape behind it.
    arch = [(398,119),(400,100),(402,96),(403,74),(405,73),(405,66),
            (408,66),(410,60),(416,58),(416,53),(421,54),(423,49),
            (428,49),(431,54),(435,52),(440,55),(441,60),(447,61),
            (447,68),(451,71),(452,92),(455,94),(457,122),
            (443,123),(442,95),(441,82),(438,75),(430,68),
            (425,66),(420,70),(415,75),(413,82),(412,96),(413,120)]
    polygon(canvas, arch, ink('283d41'))
    for face in (
        [(407,68),(410,61),(416,58),(421,56),(420,64),(413,69),(410,76),(405,76)],
        [(424,50),(428,50),(431,55),(430,62),(426,62),(422,60)],
        [(434,55),(440,56),(440,61),(443,63),(442,67),(436,64)],
        [(400,101),(404,99),(405,78),(409,78),(407,96),(405,99),(405,118),(399,120)],
        [(443,78),(445,82),(446,93),(451,96),(452,114),(447,115),(446,99),(443,96)],
    ):
        polygon(canvas, face, ink('6b766b'))
    for x, y, w in ((406,72,5),(407,80,4),(404,90,7),(403,95,7),
                     (414,63,4),(424,54,4),(443,72,5),(446,87,4),(443,99,8)):
        canvas.fill(x, y, w, 1, ink('1e343a'))
    canvas.line(420, 69, 425, 65, ink('c5b88d'))
    canvas.line(425, 65, 431, 67, ink('c5b88d'))
    # The cropped promontory and silhouettes anchor the near edge.
    ridge(canvas, [(0,115),(19,109),(58,121),(76,119),(114,133),(136,130),
                   (180,145),(210,136),(235,140),(258,136),(282,146),(314,142),
                   (341,133),(362,133),(385,121),(401,119),(410,112),(423,115),
                   (437,111),(462,116),(487,100),(512,97)], ink('243e41'))
    for x, y, h in ((14,124,43),(31,128,35),(59,130,26),(96,139,20),
                    (365,136,17),(379,132,22),(384,128,14),(468,119,22),
                    (480,117,33),(500,111,40),(509,110,51)):
        pine(canvas, x, y, h, ink('243e41'))
    for path in ([(360,144),(380,135),(389,133),(406,123),(417,123)],
                 [(380,141),(398,131),(410,131),(425,123)],
                 [(427,118),(438,115),(451,119)]):
        for p, q in zip(path, path[1:]):
            canvas.line(*p, *q, ink('4b6661'))
    # Burn down the title side like ink at the gutter of an engraved plate.
    for x in range(canvas.w):
        t = min(1.0, max(0.0, (x - 108) / 230))
        strength = (1 - t*t*(3 - 2*t)) * 0.89
        if strength:
            canvas.fill(x, 0, 1, 160, (13, 26, 32, round(255 * strength)))
    return canvas


def compass(Canvas):
    canvas = Canvas(48, 48)
    brass, gold, jade, slate = map(ink, ('a08a60', 'edd7a0', '80baa7', '192f36'))
    # The enclosing pin and four-point wind rose stay legible as a 16px glyph.
    circle = [(24 + 17.5 * math.cos(i*math.pi/8), 24 + 17.5 * math.sin(i*math.pi/8))
              for i in range(16)]
    polygon(canvas, circle, slate)
    for p, q in zip(circle, circle[1:] + circle[:1]):
        canvas.line(*p, *q, brass, 2)
    for dx, dy in ((1,-1),(-1,-1),(1,1),(-1,1)):
        polygon(canvas, [(24+dx*11,24+dy*11),(24+dx*3,24+dy*6),
                         (24,24),(24+dx*6,24+dy*3)], jade)
    for dx, dy, span in ((0,-1,23),(1,0,21),(0,1,23),(-1,0,21)):
        tip = (24+dx*span, 24+dy*span)
        left, right = (24-dy*6,24+dx*6), (24+dy*6,24-dx*6)
        polygon(canvas, [tip,left,(24,24)], gold)
        polygon(canvas, [tip,(24,24),right], brass)
    polygon(canvas, [(24,19),(29,24),(24,29),(19,24)], slate)
    polygon(canvas, [(24,21),(27,24),(24,27),(21,24)], gold)
    return canvas


def panoramic_banner(Canvas, original):
    """Reframe the same plate for a shallow phone banner.

    Hold the arch, moon and ridgeline at a uniform scale. Stretch only the
    shadow at the binding, where the gameplay labels sit.
    """
    canvas = Canvas(512, 88)
    scale = canvas.h / original.h
    gutter_end = 112
    image_start = canvas.w - (original.w - gutter_end) * scale
    for y in range(canvas.h):
        sy = min(original.h - 1, int((y + 0.5) / scale))
        for x in range(canvas.w):
            if x < image_start:
                sx = int((x + 0.5) * gutter_end / image_start)
            else:
                sx = gutter_end + int((x + 0.5 - image_start) / scale)
            offset = (sy * original.w + min(sx, original.w - 1)) * 4
            canvas.px(x, y, original.p[offset:offset+4])
    return canvas


def generate(Canvas, gui):
    atlas = panorama(Canvas)
    atlas.save(gui / 'branding/field_atlas.png')
    panoramic_banner(Canvas, atlas).save(gui / 'branding/field_atlas_wide.png')
    compass(Canvas).save(gui / 'branding/compass_seal.png')
