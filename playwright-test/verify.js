const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({
    viewport: { width: 1400, height: 1600 }
  });
  
  const reportPath = '/Users/kskim/Projects/lol-champion-simulation/report.html';
  await page.goto(`file://${reportPath}`);
  await page.waitForTimeout(1000);
  
  await page.screenshot({ path: '/Users/kskim/.gemini/antigravity/brain/003cff05-0e8c-48df-abb6-a1172da68096/default_folded_screenshot.png', fullPage: true });
  console.log('Screenshot saved to default_folded_screenshot.png');

  await browser.close();
})();
