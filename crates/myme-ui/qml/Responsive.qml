pragma Singleton
import QtQuick

QtObject {
    function columnsFor(width, cardWidth, maxCols) {
        return Math.max(1, Math.min(maxCols, Math.floor(width / cardWidth)))
    }
}
