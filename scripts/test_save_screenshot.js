const puppeteer = require('puppeteer');

(async () => {
  const browser = await puppeteer.launch({ args: ['--no-sandbox', '--disable-setuid-sandbox'] });
  const page = await browser.newPage();
  page.on('console', msg => console.log('PAGE LOG:', msg.text()));
  page.on('pageerror', err => console.log('PAGE ERROR:', err.toString()));
  
  await page.setViewport({ width: 1200, height: 800 });
  
  try {
      await page.goto('http://localhost:3000', { waitUntil: 'networkidle0' });
      await new Promise(r => setTimeout(r, 1000));
      
      const editBtn = await page.$('button[aria-label^="Edit page"]');
      if (editBtn) {
          await editBtn.click();
          await new Promise(r => setTimeout(r, 1000));
          
          await page.screenshot({ path: 'editor_open.png' });
          console.log("Screenshot editor_open.png saved");
          
          const editor = await page.$('#code-editor');
          if (editor) {
              await editor.type('\nWait Test Screenshot');
              const saveBtn = await page.$('button[aria-label="Save changes"]');
              if (saveBtn) {
                  await saveBtn.click();
                  await new Promise(r => setTimeout(r, 3000));
                  
                  await page.screenshot({ path: 'editor_saved.png' });
                  console.log("Screenshot editor_saved.png saved");
              }
          }
      }
  } catch(e) {
      console.error(e);
  }
  
  await browser.close();
})();
