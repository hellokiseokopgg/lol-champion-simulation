const { chromium } = require('playwright');
const path = require('path');

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({
    viewport: { width: 1400, height: 1600 }
  });
  
  const reportPath = path.resolve(__dirname, '../report.html');
  await page.goto(`file://${reportPath}`, { waitUntil: 'domcontentloaded' });
  await page.waitForTimeout(1000);
  
  const screenshotPath = path.resolve(__dirname, 'report_screenshot.png');
  await page.screenshot({ path: screenshotPath, fullPage: true });
  console.log('Screenshot saved to report_screenshot.png');

  await browser.close();
})();
