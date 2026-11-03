Road to Alfendrof for Mac
=========================

Requires an Apple Silicon Mac (M1 or later). Intel Macs are not supported.

1. Unzip, and drag "Road to Alfendrof" into your Applications folder (or anywhere you like).
2. The first time, macOS will say it can't verify the app (it isn't signed with an Apple developer
   account). Right-click (or Control-click) the app, choose "Open", then "Open" again. After that it
   starts normally.
   If macOS says the app is "damaged", open Terminal and run:
       xattr -cr "/Applications/Road to Alfendrof.app"
   (change the path if you put it somewhere else) and open it again.

Controls are in Options > Controls. Your settings are saved in
~/Library/Application Support/Road to Alfendrof/settings.ron

If it won't start or looks wrong, please tell me your Mac model, the macOS version, and what you see.
To get a log, run this in Terminal and send me what it prints:
    "/Applications/Road to Alfendrof.app/Contents/MacOS/t2f"
