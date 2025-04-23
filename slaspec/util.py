def addTitle(title: str, subtitle: str = "") -> str:
    top = "#" * 50
    mid = "#" * 6 + ">" + "-" * 5 + title.center(26, " ") + "-" * 5 + "<" + "#" * 6
    bot = top if not subtitle else (" " + subtitle + " ").center(50, "#")

    return "\n".join((top, mid, bot))
