use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::value;
use nom::{IResult, Parser};

pub const ALICEBLUE: [u8; 3] = [240, 248, 255];
pub const ANTIQUEWHITE: [u8; 3] = [250, 235, 215];
pub const AQUA: [u8; 3] = [0, 255, 255];
pub const AQUAMARINE: [u8; 3] = [127, 255, 212];
pub const AZURE: [u8; 3] = [240, 255, 255];
pub const BEIGE: [u8; 3] = [245, 245, 220];
pub const BISQUE: [u8; 3] = [255, 228, 196];
pub const BLACK: [u8; 3] = [0, 0, 0];
pub const BLANCHEDALMOND: [u8; 3] = [255, 235, 205];
pub const BLUE: [u8; 3] = [0, 0, 255];
pub const BLUEVIOLET: [u8; 3] = [138, 43, 226];
pub const BROWN: [u8; 3] = [165, 42, 42];
pub const BURLYWOOD: [u8; 3] = [222, 184, 135];
pub const CADETBLUE: [u8; 3] = [95, 158, 160];
pub const CHARTREUSE: [u8; 3] = [127, 255, 0];
pub const CHOCOLATE: [u8; 3] = [210, 105, 30];
pub const CORAL: [u8; 3] = [255, 127, 80];
pub const CORNFLOWERBLUE: [u8; 3] = [100, 149, 237];
pub const CORNSILK: [u8; 3] = [255, 248, 220];
pub const CRIMSON: [u8; 3] = [220, 20, 60];
pub const CYAN: [u8; 3] = [0, 255, 255];
pub const DARKBLUE: [u8; 3] = [0, 0, 139];
pub const DARKCYAN: [u8; 3] = [0, 139, 139];
pub const DARKGOLDENROD: [u8; 3] = [184, 134, 11];
pub const DARKGRAY: [u8; 3] = [169, 169, 169];
pub const DARKGREEN: [u8; 3] = [0, 100, 0];
pub const DARKGREY: [u8; 3] = [169, 169, 169];
pub const DARKKHAKI: [u8; 3] = [189, 183, 107];
pub const DARKMAGENTA: [u8; 3] = [139, 0, 139];
pub const DARKOLIVEGREEN: [u8; 3] = [85, 107, 47];
pub const DARKORANGE: [u8; 3] = [255, 140, 0];
pub const DARKORCHID: [u8; 3] = [153, 50, 204];
pub const DARKRED: [u8; 3] = [139, 0, 0];
pub const DARKSALMON: [u8; 3] = [233, 150, 122];
pub const DARKSEAGREEN: [u8; 3] = [143, 188, 143];
pub const DARKSLATEBLUE: [u8; 3] = [72, 61, 139];
pub const DARKSLATEGRAY: [u8; 3] = [47, 79, 79];
pub const DARKSLATEGREY: [u8; 3] = [47, 79, 79];
pub const DARKTURQUOISE: [u8; 3] = [0, 206, 209];
pub const DARKVIOLET: [u8; 3] = [148, 0, 211];
pub const DEEPPINK: [u8; 3] = [255, 20, 147];
pub const DEEPSKYBLUE: [u8; 3] = [0, 191, 255];
pub const DIMGRAY: [u8; 3] = [105, 105, 105];
pub const DIMGREY: [u8; 3] = [105, 105, 105];
pub const DODGERBLUE: [u8; 3] = [30, 144, 255];
pub const FIREBRICK: [u8; 3] = [178, 34, 34];
pub const FLORALWHITE: [u8; 3] = [255, 250, 240];
pub const FORESTGREEN: [u8; 3] = [34, 139, 34];
pub const FUCHSIA: [u8; 3] = [255, 0, 255];
pub const GAINSBORO: [u8; 3] = [220, 220, 220];
pub const GHOSTWHITE: [u8; 3] = [248, 248, 255];
pub const GOLD: [u8; 3] = [255, 215, 0];
pub const GOLDENROD: [u8; 3] = [218, 165, 32];
pub const GRAY: [u8; 3] = [128, 128, 128];
pub const GREY: [u8; 3] = [128, 128, 128];
pub const GREEN: [u8; 3] = [0, 128, 0];
pub const GREENYELLOW: [u8; 3] = [173, 255, 47];
pub const HONEYDEW: [u8; 3] = [240, 255, 240];
pub const HOTPINK: [u8; 3] = [255, 105, 180];
pub const INDIANRED: [u8; 3] = [205, 92, 92];
pub const INDIGO: [u8; 3] = [75, 0, 130];
pub const IVORY: [u8; 3] = [255, 255, 240];
pub const KHAKI: [u8; 3] = [240, 230, 140];
pub const LAVENDER: [u8; 3] = [230, 230, 250];
pub const LAVENDERBLUSH: [u8; 3] = [255, 240, 245];
pub const LAWNGREEN: [u8; 3] = [124, 252, 0];
pub const LEMONCHIFFON: [u8; 3] = [255, 250, 205];
pub const LIGHTBLUE: [u8; 3] = [173, 216, 230];
pub const LIGHTCORAL: [u8; 3] = [240, 128, 128];
pub const LIGHTCYAN: [u8; 3] = [224, 255, 255];
pub const LIGHTGOLDENRODYELLOW: [u8; 3] = [250, 250, 210];
pub const LIGHTGRAY: [u8; 3] = [211, 211, 211];
pub const LIGHTGREEN: [u8; 3] = [144, 238, 144];
pub const LIGHTGREY: [u8; 3] = [211, 211, 211];
pub const LIGHTPINK: [u8; 3] = [255, 182, 193];
pub const LIGHTSALMON: [u8; 3] = [255, 160, 122];
pub const LIGHTSEAGREEN: [u8; 3] = [32, 178, 170];
pub const LIGHTSKYBLUE: [u8; 3] = [135, 206, 250];
pub const LIGHTSLATEGRAY: [u8; 3] = [119, 136, 153];
pub const LIGHTSLATEGREY: [u8; 3] = [119, 136, 153];
pub const LIGHTSTEELBLUE: [u8; 3] = [176, 196, 222];
pub const LIGHTYELLOW: [u8; 3] = [255, 255, 224];
pub const LIME: [u8; 3] = [0, 255, 0];
pub const LIMEGREEN: [u8; 3] = [50, 205, 50];
pub const LINEN: [u8; 3] = [250, 240, 230];
pub const MAGENTA: [u8; 3] = [255, 0, 255];
pub const MAROON: [u8; 3] = [128, 0, 0];
pub const MEDIUMAQUAMARINE: [u8; 3] = [102, 205, 170];
pub const MEDIUMBLUE: [u8; 3] = [0, 0, 205];
pub const MEDIUMORCHID: [u8; 3] = [186, 85, 211];
pub const MEDIUMPURPLE: [u8; 3] = [147, 112, 219];
pub const MEDIUMSEAGREEN: [u8; 3] = [60, 179, 113];
pub const MEDIUMSLATEBLUE: [u8; 3] = [123, 104, 238];
pub const MEDIUMSPRINGGREEN: [u8; 3] = [0, 250, 154];
pub const MEDIUMTURQUOISE: [u8; 3] = [72, 209, 204];
pub const MEDIUMVIOLETRED: [u8; 3] = [199, 21, 133];
pub const MIDNIGHTBLUE: [u8; 3] = [25, 25, 112];
pub const MINTCREAM: [u8; 3] = [245, 255, 250];
pub const MISTYROSE: [u8; 3] = [255, 228, 225];
pub const MOCCASIN: [u8; 3] = [255, 228, 181];
pub const NAVAJOWHITE: [u8; 3] = [255, 222, 173];
pub const NAVY: [u8; 3] = [0, 0, 128];
pub const OLDLACE: [u8; 3] = [253, 245, 230];
pub const OLIVE: [u8; 3] = [128, 128, 0];
pub const OLIVEDRAB: [u8; 3] = [107, 142, 35];
pub const ORANGE: [u8; 3] = [255, 165, 0];
pub const ORANGERED: [u8; 3] = [255, 69, 0];
pub const ORCHID: [u8; 3] = [218, 112, 214];
pub const PALEGOLDENROD: [u8; 3] = [238, 232, 170];
pub const PALEGREEN: [u8; 3] = [152, 251, 152];
pub const PALETURQUOISE: [u8; 3] = [175, 238, 238];
pub const PALEVIOLETRED: [u8; 3] = [219, 112, 147];
pub const PAPAYAWHIP: [u8; 3] = [255, 239, 213];
pub const PEACHPUFF: [u8; 3] = [255, 218, 185];
pub const PERU: [u8; 3] = [205, 133, 63];
pub const PINK: [u8; 3] = [255, 192, 203];
pub const PLUM: [u8; 3] = [221, 160, 221];
pub const POWDERBLUE: [u8; 3] = [176, 224, 230];
pub const PURPLE: [u8; 3] = [128, 0, 128];
pub const REBECCAPURPLE: [u8; 3] = [102, 51, 153];
pub const RED: [u8; 3] = [255, 0, 0];
pub const ROSYBROWN: [u8; 3] = [188, 143, 143];
pub const ROYALBLUE: [u8; 3] = [65, 105, 225];
pub const SADDLEBROWN: [u8; 3] = [139, 69, 19];
pub const SALMON: [u8; 3] = [250, 128, 114];
pub const SANDYBROWN: [u8; 3] = [244, 164, 96];
pub const SEAGREEN: [u8; 3] = [46, 139, 87];
pub const SEASHELL: [u8; 3] = [255, 245, 238];
pub const SIENNA: [u8; 3] = [160, 82, 45];
pub const SILVER: [u8; 3] = [192, 192, 192];
pub const SKYBLUE: [u8; 3] = [135, 206, 235];
pub const SLATEBLUE: [u8; 3] = [106, 90, 205];
pub const SLATEGRAY: [u8; 3] = [112, 128, 144];
pub const SLATEGREY: [u8; 3] = [112, 128, 144];
pub const SNOW: [u8; 3] = [255, 250, 250];
pub const SPRINGGREEN: [u8; 3] = [0, 255, 127];
pub const STEELBLUE: [u8; 3] = [70, 130, 180];
pub const TAN: [u8; 3] = [210, 180, 140];
pub const TEAL: [u8; 3] = [0, 128, 128];
pub const THISTLE: [u8; 3] = [216, 191, 216];
pub const TOMATO: [u8; 3] = [255, 99, 71];
pub const TURQUOISE: [u8; 3] = [64, 224, 208];
pub const VIOLET: [u8; 3] = [238, 130, 238];
pub const WHEAT: [u8; 3] = [245, 222, 179];
pub const WHITE: [u8; 3] = [255, 255, 255];
pub const WHITESMOKE: [u8; 3] = [245, 245, 245];
pub const YELLOW: [u8; 3] = [255, 255, 0];
pub const YELLOWGREEN: [u8; 3] = [154, 205, 50];

pub fn color(input: &str) -> IResult<&str, [u8; 3]> {
    alt((
        alt((
            value(ALICEBLUE, tag("ALICEBLUE")),
            value(ANTIQUEWHITE, tag("ANTIQUEWHITE")),
            value(AQUA, tag("AQUA")),
            value(AQUAMARINE, tag("AQUAMARINE")),
            value(AZURE, tag("AZURE")),
            value(BEIGE, tag("BEIGE")),
            value(BISQUE, tag("BISQUE")),
            value(BLACK, tag("BLACK")),
            value(BLANCHEDALMOND, tag("BLANCHEDALMOND")),
            value(BLUE, tag("BLUE")),
            value(BLUEVIOLET, tag("BLUEVIOLET")),
            value(BROWN, tag("BROWN")),
            value(BURLYWOOD, tag("BURLYWOOD")),
            value(CADETBLUE, tag("CADETBLUE")),
            value(CHARTREUSE, tag("CHARTREUSE")),
            value(CHOCOLATE, tag("CHOCOLATE")),
            value(CORAL, tag("CORAL")),
            value(CORNFLOWERBLUE, tag("CORNFLOWERBLUE")),
            value(CORNSILK, tag("CORNSILK")),
            value(CRIMSON, tag("CRIMSON")),
            value(CYAN, tag("CYAN")),
        )),
        alt((
            value(DARKBLUE, tag("DARKBLUE")),
            value(DARKCYAN, tag("DARKCYAN")),
            value(DARKGOLDENROD, tag("DARKGOLDENROD")),
            value(DARKGRAY, tag("DARKGRAY")),
            value(DARKGREEN, tag("DARKGREEN")),
            value(DARKGREY, tag("DARKGREY")),
            value(DARKKHAKI, tag("DARKKHAKI")),
            value(DARKMAGENTA, tag("DARKMAGENTA")),
            value(DARKOLIVEGREEN, tag("DARKOLIVEGREEN")),
            value(DARKORANGE, tag("DARKORANGE")),
            value(DARKORCHID, tag("DARKORCHID")),
            value(DARKRED, tag("DARKRED")),
            value(DARKSALMON, tag("DARKSALMON")),
            value(DARKSEAGREEN, tag("DARKSEAGREEN")),
            value(DARKSLATEBLUE, tag("DARKSLATEBLUE")),
            value(DARKSLATEGRAY, tag("DARKSLATEGRAY")),
            value(DARKSLATEGREY, tag("DARKSLATEGREY")),
            value(DARKTURQUOISE, tag("DARKTURQUOISE")),
            value(DARKVIOLET, tag("DARKVIOLET")),
            value(DEEPPINK, tag("DEEPPINK")),
            value(DEEPSKYBLUE, tag("DEEPSKYBLUE")),
        )),
        alt((
            value(DIMGRAY, tag("DIMGRAY")),
            value(DIMGREY, tag("DIMGREY")),
            value(DODGERBLUE, tag("DODGERBLUE")),
            value(FIREBRICK, tag("FIREBRICK")),
            value(FLORALWHITE, tag("FLORALWHITE")),
            value(FORESTGREEN, tag("FORESTGREEN")),
            value(FUCHSIA, tag("FUCHSIA")),
            value(GAINSBORO, tag("GAINSBORO")),
            value(GHOSTWHITE, tag("GHOSTWHITE")),
            value(GOLD, tag("GOLD")),
            value(GOLDENROD, tag("GOLDENROD")),
            value(GRAY, tag("GRAY")),
            value(GREY, tag("GREY")),
            value(GREEN, tag("GREEN")),
            value(GREENYELLOW, tag("GREENYELLOW")),
            value(HONEYDEW, tag("HONEYDEW")),
            value(HOTPINK, tag("HOTPINK")),
            value(INDIANRED, tag("INDIANRED")),
            value(INDIGO, tag("INDIGO")),
            value(IVORY, tag("IVORY")),
            value(KHAKI, tag("KHAKI")),
        )),
        alt((
            value(LAVENDER, tag("LAVENDER")),
            value(LAVENDERBLUSH, tag("LAVENDERBLUSH")),
            value(LAWNGREEN, tag("LAWNGREEN")),
            value(LEMONCHIFFON, tag("LEMONCHIFFON")),
            value(LIGHTBLUE, tag("LIGHTBLUE")),
            value(LIGHTCORAL, tag("LIGHTCORAL")),
            value(LIGHTCYAN, tag("LIGHTCYAN")),
            value(LIGHTGOLDENRODYELLOW, tag("LIGHTGOLDENRODYELLOW")),
            value(LIGHTGRAY, tag("LIGHTGRAY")),
            value(LIGHTGREEN, tag("LIGHTGREEN")),
            value(LIGHTGREY, tag("LIGHTGREY")),
            value(LIGHTPINK, tag("LIGHTPINK")),
            value(LIGHTSALMON, tag("LIGHTSALMON")),
            value(LIGHTSEAGREEN, tag("LIGHTSEAGREEN")),
            value(LIGHTSKYBLUE, tag("LIGHTSKYBLUE")),
            value(LIGHTSLATEGRAY, tag("LIGHTSLATEGRAY")),
            value(LIGHTSLATEGREY, tag("LIGHTSLATEGREY")),
            value(LIGHTSTEELBLUE, tag("LIGHTSTEELBLUE")),
            value(LIGHTYELLOW, tag("LIGHTYELLOW")),
            value(LIME, tag("LIME")),
            value(LIMEGREEN, tag("LIMEGREEN")),
        )),
        alt((
            value(LINEN, tag("LINEN")),
            value(MAGENTA, tag("MAGENTA")),
            value(MAROON, tag("MAROON")),
            value(MEDIUMAQUAMARINE, tag("MEDIUMAQUAMARINE")),
            value(MEDIUMBLUE, tag("MEDIUMBLUE")),
            value(MEDIUMORCHID, tag("MEDIUMORCHID")),
            value(MEDIUMPURPLE, tag("MEDIUMPURPLE")),
            value(MEDIUMSEAGREEN, tag("MEDIUMSEAGREEN")),
            value(MEDIUMSLATEBLUE, tag("MEDIUMSLATEBLUE")),
            value(MEDIUMSPRINGGREEN, tag("MEDIUMSPRINGGREEN")),
            value(MEDIUMTURQUOISE, tag("MEDIUMTURQUOISE")),
            value(MEDIUMVIOLETRED, tag("MEDIUMVIOLETRED")),
            value(MIDNIGHTBLUE, tag("MIDNIGHTBLUE")),
            value(MINTCREAM, tag("MINTCREAM")),
            value(MISTYROSE, tag("MISTYROSE")),
            value(MOCCASIN, tag("MOCCASIN")),
            value(NAVAJOWHITE, tag("NAVAJOWHITE")),
            value(NAVY, tag("NAVY")),
            value(OLDLACE, tag("OLDLACE")),
            value(OLIVE, tag("OLIVE")),
            value(OLIVEDRAB, tag("OLIVEDRAB")),
        )),
        alt((
            value(ORANGE, tag("ORANGE")),
            value(ORANGERED, tag("ORANGERED")),
            value(ORCHID, tag("ORCHID")),
            value(PALEGOLDENROD, tag("PALEGOLDENROD")),
            value(PALEGREEN, tag("PALEGREEN")),
            value(PALETURQUOISE, tag("PALETURQUOISE")),
            value(PALEVIOLETRED, tag("PALEVIOLETRED")),
            value(PAPAYAWHIP, tag("PAPAYAWHIP")),
            value(PEACHPUFF, tag("PEACHPUFF")),
            value(PERU, tag("PERU")),
            value(PINK, tag("PINK")),
            value(PLUM, tag("PLUM")),
            value(POWDERBLUE, tag("POWDERBLUE")),
            value(PURPLE, tag("PURPLE")),
            value(REBECCAPURPLE, tag("REBECCAPURPLE")),
            value(RED, tag("RED")),
            value(ROSYBROWN, tag("ROSYBROWN")),
            value(ROYALBLUE, tag("ROYALBLUE")),
            value(SADDLEBROWN, tag("SADDLEBROWN")),
            value(SALMON, tag("SALMON")),
            value(SANDYBROWN, tag("SANDYBROWN")),
        )),
        alt((
            value(SEAGREEN, tag("SEAGREEN")),
            value(SEASHELL, tag("SEASHELL")),
            value(SIENNA, tag("SIENNA")),
            value(SILVER, tag("SILVER")),
            value(SKYBLUE, tag("SKYBLUE")),
            value(SLATEBLUE, tag("SLATEBLUE")),
            value(SLATEGRAY, tag("SLATEGRAY")),
            value(SLATEGREY, tag("SLATEGREY")),
            value(SNOW, tag("SNOW")),
            value(SPRINGGREEN, tag("SPRINGGREEN")),
            value(STEELBLUE, tag("STEELBLUE")),
            value(TAN, tag("TAN")),
            value(TEAL, tag("TEAL")),
            value(THISTLE, tag("THISTLE")),
            value(TOMATO, tag("TOMATO")),
            value(TURQUOISE, tag("TURQUOISE")),
            value(VIOLET, tag("VIOLET")),
            value(WHEAT, tag("WHEAT")),
            value(WHITE, tag("WHITE")),
            value(WHITESMOKE, tag("WHITESMOKE")),
            value(YELLOW, tag("YELLOW")),
        )),
        value(YELLOWGREEN, tag("YELLOWGREEN")),
    ))
    .parse(input)
}
