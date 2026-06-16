import urllib.request
import json

# OP.GG Build Tab API often looks like:
# https://lol-web-api.op.gg/api/v1.0/internal/bypass/matches/kr/TRhLWfA2oBH15y7FZh5ItrOTtY1sG4sgiAsd1ODIrBo%3D
# Or it could be that the Build Tab is just a React Server Component fetch.
# Let's search the internet for OP.GG match build api.
