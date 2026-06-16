const { chromium } = require('playwright');
const path = require('path');

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({
    viewport: { width: 1400, height: 1600 }
  });
  
  // Get absolute path to report.html
  const reportPath = '/Users/kskim/Projects/lol-champion-simulation/report.html';
  await page.goto(`file://${reportPath}`);
  
  // Wait for rendering
  await page.waitForTimeout(1000);
  
  await page.screenshot({ path: '/Users/kskim/.gemini/antigravity/brain/003cff05-0e8c-48df-abb6-a1172da68096/full_page_screenshot.png', fullPage: true });
  console.log('Screenshot saved to full_page_screenshot.png');

  await browser.close();
})();
