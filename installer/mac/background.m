// Render installer artwork with macOS AppKit; no downloaded art or third-party tools.
#import <AppKit/AppKit.h>

static NSColor *ink(void) {
    return [NSColor colorWithCalibratedRed:0.22 green:0.13 blue:0.31 alpha:1];
}

static void centered(NSString *text, CGFloat y, CGFloat size, BOOL bold) {
    NSMutableParagraphStyle *style = [[NSMutableParagraphStyle alloc] init];
    style.alignment = NSTextAlignmentCenter;
    NSDictionary *attributes = @{
        NSFontAttributeName: bold ? [NSFont boldSystemFontOfSize:size] : [NSFont systemFontOfSize:size],
        NSForegroundColorAttributeName: ink(),
        NSParagraphStyleAttributeName: style,
    };
    [text drawInRect:NSMakeRect(20, y, 760, 30) withAttributes:attributes];
}

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc != 4) {
            fprintf(stderr, "usage: background LOGO EMBLEM OUTPUT.png\n");
            return 1;
        }
        NSImage *logo = [[NSImage alloc] initWithContentsOfFile:@(argv[1])];
        NSImage *emblem = [[NSImage alloc] initWithContentsOfFile:@(argv[2])];
        NSBitmapImageRep *bitmap = [[NSBitmapImageRep alloc]
            initWithBitmapDataPlanes:NULL pixelsWide:800 pixelsHigh:520
            bitsPerSample:8 samplesPerPixel:4 hasAlpha:YES isPlanar:NO
            colorSpaceName:NSDeviceRGBColorSpace bytesPerRow:0 bitsPerPixel:0];
        NSGraphicsContext *context = [NSGraphicsContext graphicsContextWithBitmapImageRep:bitmap];
        if (!logo || !emblem || !bitmap || !context) {
            fprintf(stderr, "Unable to load game artwork or create installer background.\n");
            return 1;
        }
        [NSGraphicsContext saveGraphicsState];
        [NSGraphicsContext setCurrentContext:context];
        [[NSColor colorWithCalibratedRed:0.95 green:0.93 blue:0.88 alpha:1] setFill];
        NSRectFill(NSMakeRect(0, 0, 800, 520));
        [ink() setFill];
        NSRectFill(NSMakeRect(0, 356, 800, 164));
        context.imageInterpolation = NSImageInterpolationNone;
        [logo drawInRect:NSMakeRect(200, 384, 400, 124)];
        [emblem drawInRect:NSMakeRect(42, 412, 56, 56)];
        [emblem drawInRect:NSMakeRect(702, 412, 56, 56)];
        centered(@"Drag the game to Applications", 312, 23, YES);
        NSBezierPath *arrow = [NSBezierPath bezierPath];
        [arrow moveToPoint:NSMakePoint(334, 251)];
        [arrow lineToPoint:NSMakePoint(464, 251)];
        [arrow moveToPoint:NSMakePoint(446, 267)];
        [arrow lineToPoint:NSMakePoint(464, 251)];
        [arrow lineToPoint:NSMakePoint(446, 235)];
        arrow.lineWidth = 5;
        [[NSColor colorWithCalibratedRed:0.50 green:0.30 blue:0.61 alpha:1] setStroke];
        [arrow stroke];
        centered(@"Read README before your first launch", 53, 17, YES);
        centered(@"Eject this disk after copying • Launch the installed app", 20, 14, NO);
        [NSGraphicsContext restoreGraphicsState];
        NSData *data = [bitmap representationUsingType:NSBitmapImageFileTypePNG properties:@{}];
        NSError *error = nil;
        if (!data || ![data writeToFile:@(argv[3]) options:NSDataWritingAtomic error:&error]) {
            fprintf(stderr, "Unable to write installer background: %s\n", error.localizedDescription.UTF8String ?: "PNG encoding failed");
            return 1;
        }
    }
    return 0;
}
