from pathlib import Path

from slaspec.builder import SLASpecBuilder

if __name__ == "__main__":
    cwd = Path("sleigh")
    slaspec = SLASpecBuilder()
    slaspec.build(cwd)
