const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const reportsToTest = [
  'report.html',
  'report_garen_darius.html',
  'report_zed_dummy.html'
];

const viewports = [
  { name: 'Desktop (1400px)', width: 1400, height: 1200 },
  { name: 'Laptop/Tablet Land (1024px)', width: 1024, height: 1000 },
  { name: 'Tablet Port (768px)', width: 768, height: 900 },
  { name: 'Mobile (480px)', width: 480, height: 800 }
];

(async () => {
  const browser = await chromium.launch();
  
  for (const reportFile of reportsToTest) {
    const reportPath = path.resolve(__dirname, '..', reportFile);
    if (!fs.existsSync(reportPath)) {
      console.warn(`⚠️ Warning: ${reportFile} does not exist, skipping.`);
      continue;
    }
    
    console.log(`\n==================================================`);
    console.log(`Testing report file: ${reportFile}`);
    console.log(`==================================================`);

    const htmlContent = fs.readFileSync(reportPath, 'utf8');

    // 1. Inspect generated report.html's javascript variables to ensure they are valid JSON
    console.log('--- 1. Validating Javascript variables ---');

    function extractVarValue(html, varName) {
      const regex = new RegExp(`const\\s+${varName}\\s*=\\s*([\\s\\S]*?);\\s*(?:const|let|var|//|$)`, 'm');
      const match = html.match(regex);
      if (!match) {
        throw new Error(`Variable ${varName} not found in ${reportFile}`);
      }
      return match[1].trim();
    }

    const varsToValidate = ['events', 'itemsData', 'runesData', 'runeTreesData', 'statsData'];
    const parsedData = {};

    for (const varName of varsToValidate) {
      try {
        const rawValue = extractVarValue(htmlContent, varName);
        const parsed = JSON.parse(rawValue);
        console.log(`✅ Variable "${varName}" successfully parsed as valid JSON.`);
        parsedData[varName] = parsed;
      } catch (err) {
        console.error(`❌ Failed to parse variable "${varName}":`, err.message);
        process.exit(1);
      }
    }

    // Additional manual validations on the parsed JSON structures
    console.log('--- 2. Validating JSON contents and properties ---');

    if (!Array.isArray(parsedData.events)) {
      console.error('❌ "events" should be an array');
      process.exit(1);
    }
    console.log(`✅ "events" is an array with ${parsedData.events.length} elements.`);

    for (const key of ['itemsData', 'runesData', 'statsData']) {
      if (typeof parsedData[key] !== 'object' || parsedData[key] === null) {
        console.error(`❌ "${key}" should be a non-null object`);
        process.exit(1);
      }
      console.log(`✅ "${key}" is a valid object.`);
    }

    const champions = Object.keys(parsedData.statsData);
    console.log('Champions found in statsData:', champions);
    if (champions.length < 2) {
      console.error('❌ statsData should contain stats for at least 2 champions.');
      process.exit(1);
    }

    // 2. Playwright execution to check for runtime JS errors & UI stability across viewports
    console.log('--- 3. Launching Playwright to verify page execution & responsive layout ---');
    
    for (const vp of viewports) {
      console.log(`\n  Checking viewport: ${vp.name}`);
      const page = await browser.newPage({
        viewport: { width: vp.width, height: vp.height }
      });

      const errors = [];
      page.on('pageerror', (err) => {
        console.error('  ❌ JS Page Error detected:', err.message);
        errors.push(err.message);
      });
      page.on('console', (msg) => {
        if (msg.type() === 'error') {
          console.error('  ❌ Console Error detected:', msg.text());
          errors.push(msg.text());
        }
      });

      await page.goto(`file://${reportPath}`, { waitUntil: 'domcontentloaded' });
      await page.waitForTimeout(500);

      // Take screenshot of the rendered page for this viewport
      const screenshotDir = '/Users/kskim/Projects/lol-champion-simulation/.agents/challenger_stats_2/screenshots';
      if (!fs.existsSync(screenshotDir)) {
        fs.mkdirSync(screenshotDir, { recursive: true });
      }
      const vpNameClean = vp.name.replace(/[^a-zA-Z0-9]/g, '_').toLowerCase();
      const screenshotPath = path.resolve(screenshotDir, `stats_${reportFile.replace('.html', '')}_${vpNameClean}.png`);
      await page.screenshot({ path: screenshotPath, fullPage: true });
      console.log(`  ✅ Screenshot saved to ${screenshotPath}`);

      // Check if starting stats panels are rendered
      const startingStatsCount = await page.$$eval('div', (divs) => {
        return divs.filter(d => d.innerText && d.innerText.includes('Starting Stats')).length;
      });

      if (startingStatsCount < 2) {
        errors.push(`Expected at least 2 "Starting Stats" panels, but found ${startingStatsCount}`);
      }

      // Verify that the layout of itemsContainer is flexible and fits well
      const containerStyle = await page.$eval('#itemsContainer', (el) => {
        const rect = el.getBoundingClientRect();
        const style = window.getComputedStyle(el);
        return {
          display: style.display,
          flexDirection: style.flexDirection,
          gap: style.gap,
          width: rect.width,
          height: rect.height,
          childrenCount: el.children.length
        };
      });

      console.log('  itemsContainer layout details:', containerStyle);
      if (containerStyle.display !== 'flex') {
        errors.push(`Expected #itemsContainer display to be "flex", got: ${containerStyle.display}`);
      }
      if (containerStyle.childrenCount < 2) {
        errors.push(`Expected at least 2 champion columns inside #itemsContainer, got: ${containerStyle.childrenCount}`);
      }

      // Check individual columns width and spacing
      const colsDetails = await page.$$eval('#itemsContainer > .col', (cols) => {
        return cols.map((col, index) => {
          const rect = col.getBoundingClientRect();
          const style = window.getComputedStyle(col);
          
          // Look for stats panel within the column
          const statsPanel = col.querySelector('div[style*="grid-template-columns"]');
          const statsPanelRect = statsPanel ? statsPanel.getBoundingClientRect() : null;
          
          return {
            index,
            width: rect.width,
            height: rect.height,
            left: rect.left,
            right: rect.right,
            hasStatsPanel: !!statsPanel,
            statsPanelWidth: statsPanelRect ? statsPanelRect.width : 0,
            statsPanelHeight: statsPanelRect ? statsPanelRect.height : 0
          };
        });
      });

      console.log('  Columns layout details:', colsDetails);
      
      if (colsDetails.length >= 2) {
        const col0 = colsDetails[0];
        const col1 = colsDetails[1];
        
        // Check if columns overlap
        if (col0.right > col1.left) {
          errors.push(`Layout Overlap Detected: Column 0 right edge (${col0.right}px) is beyond Column 1 left edge (${col1.left}px)`);
        } else {
          console.log('  ✅ No overlap detected between champion columns.');
        }

        // Check if stats panels are present in both columns
        if (!col0.hasStatsPanel || !col1.hasStatsPanel) {
          errors.push('❌ Missing starting stats panel inside one of the champion columns.');
        } else {
          console.log('  ✅ Stats panels are present inside both columns.');
        }

        // Check if the stats panel fits inside the column width
        if (col0.statsPanelWidth > col0.width + 1 || col1.statsPanelWidth > col1.width + 1) {
          errors.push(`❌ Stats panel width (Col0: ${col0.statsPanelWidth}px, Col1: ${col1.statsPanelWidth}px) exceeds column width (Col0: ${col0.width}px, Col1: ${col1.width}px)`);
        } else {
          console.log(`  ✅ Stats panel width (${col0.statsPanelWidth}px) fits within column width (${col0.width}px).`);
        }
      }

      await page.close();

      if (errors.length > 0) {
        console.error(`  ❌ Verification FAILED for viewport ${vp.name} with errors:`);
        console.error(JSON.stringify(errors, null, 2));
        process.exit(1);
      } else {
        console.log(`  ✅ Viewport check passed.`);
      }
    }

    console.log(`✅ SUCCESS: All checks passed for ${reportFile}!`);
  }

  await browser.close();
  console.log('\n🎉 ALL REPORT FILES AND VIEWPORTS VERIFIED SUCCESSFULLY!');
  process.exit(0);
})();
