const puppeteer = require('puppeteer');

(async () => {
  const browser = await puppeteer.launch({ args: ['--no-sandbox', '--disable-setuid-sandbox'] });
  const page = await browser.newPage();
  
  // Set viewport to mobile size (iPhone X)
  await page.setViewport({ width: 375, height: 812, isMobile: true, hasTouch: true });
  
  try {
      await page.goto('http://localhost:3000', { waitUntil: 'networkidle0' });
      await new Promise(r => setTimeout(r, 1000));
      
      await page.screenshot({ path: 'mobile_view.png' });
      console.log("Screenshot mobile_view.png saved");
      
      const editBtn = await page.$('button[aria-label^="Edit page"]');
      if (editBtn) {
          console.log("Found edit button, clicking...");
          await editBtn.click();
          await new Promise(r => setTimeout(r, 1000));
          
          await page.screenshot({ path: 'mobile_edit.png' });
          console.log("Screenshot mobile_edit.png saved");
      }
  } catch(e) {
      console.error(e);
  }
  
  await browser.close();
})();
