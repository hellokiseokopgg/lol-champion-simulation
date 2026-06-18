const { chromium } = require('playwright');
const path = require('path');

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({
    viewport: { width: 1400, height: 1600 }
  });

  page.on('pageerror', exception => {
    console.error(`JSERROR: Page error: ${exception.message}`);
    process.exit(1);
  });
  
  page.on('console', msg => {
    if (msg.type() === 'error') {
      console.error(`JSERROR: Console error: ${msg.text()}`);
    }
  });
  
  const reportPath = path.resolve(__dirname, '../report.html');
  await page.goto(`file://${reportPath}`, { waitUntil: 'load' });
  await page.waitForTimeout(2000);
  
  const screenshotPath = path.resolve(__dirname, 'report_screenshot.png');
  await page.screenshot({ path: screenshotPath, fullPage: true });
  console.log('Screenshot saved to report_screenshot.png');

  await browser.close();
})();
