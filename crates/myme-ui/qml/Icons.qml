pragma Singleton
import QtQuick

QtObject {
    id: icons

    // Font loader for Phosphor Icons
    property FontLoader phosphor: FontLoader {
        source: "fonts/Phosphor.ttf"
    }

    // Font family name
    readonly property string family: phosphor.name

    // Navigation
    readonly property string house: "\ueb9c"
    readonly property string note: "\uec11"
    readonly property string notePencil: "\uec15"
    readonly property string folder: "\ueb21"
    readonly property string folderSimple: "\ueb2c"
    readonly property string gearSix: "\ueb45"
    readonly property string wrench: "\uedda"
    readonly property string code: "\uea58"
    readonly property string terminalWindow: "\ued55"
    readonly property string squaresFour: "\ued20"
    readonly property string list: "\uebca"
    readonly property string sidebar: "\uece5"
    readonly property string sidebarSimple: "\uece6"

    // Arrows & Carets
    readonly property string caretLeft: "\ue9ff"
    readonly property string caretRight: "\uea00"
    readonly property string caretUp: "\uea01"
    readonly property string caretDown: "\ue9fe"
    readonly property string arrowsClockwise: "\ue95d"

    // Actions
    readonly property string x: "\ueddb"
    readonly property string xCircle: "\ueddc"
    readonly property string minus: "\uebf8"
    readonly property string plus: "\uec86"

    // Window controls
    readonly property string square: "\ued1c"
    readonly property string cornersOut: "\uea70"
    readonly property string cornersIn: "\uea6f"
    readonly property string check: "\uea30"
    readonly property string copy: "\uea6b"
    readonly property string trash: "\ued8a"
    readonly property string trashSimple: "\ued8b"
    readonly property string pencil: "\uec56"
    readonly property string pencilSimple: "\uec59"
    readonly property string signOut: "\uecea"

    // Theme
    readonly property string sun: "\ued3e"
    readonly property string moon: "\uebfe"
    readonly property string circleHalf: "\uea39"

    // Weather
    readonly property string cloud: "\uea55"
    readonly property string cloud_sun: "\uea5e"
    readonly property string cloud_fog: "\uea57"
    readonly property string cloud_rain: "\uea5a"
    readonly property string cloud_snow: "\uea5c"
    readonly property string cloud_lightning: "\uea59"
    readonly property string thermometer: "\ued57"
    readonly property string drop: "\ueab4"
    readonly property string wind: "\uedcb"

    // Status
    readonly property string warning: "\uedbf"
    readonly property string info: "\ueba7"
    readonly property string spinner: "\ued16"
    readonly property string dotsThree: "\ueaaf"

    // Auth & Security
    readonly property string gitBranch: "\ueb4f"
    readonly property string githubLogo: "\ueb53"
    readonly property string key: "\uebae"
    readonly property string lock: "\uebd1"
    readonly property string user: "\ueda0"

    // Text & Documents
    readonly property string scissors: "\uecb8"
    readonly property string textT: "\ued5c"
    readonly property string article: "\ue958"

    // Email
    readonly property string envelopeSimple: "\uead2"
    readonly property string envelopeOpen: "\ueacd"
    readonly property string star: "\ued29"
    readonly property string starFill: "\ued28"

    // Calendar
    readonly property string calendarBlank: "\ue9d0"
    readonly property string calendarCheck: "\ue9d5"
    readonly property string clock: "\uea52"
    readonly property string mapPin: "\uebe1"
}
