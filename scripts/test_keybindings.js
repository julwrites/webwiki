const puppeteer = require('puppeteer');

(async () => {
  const browser = await puppeteer.launch({ args: ['--no-sandbox', '--disable-setuid-sandbox'] });
  const page = await browser.newPage();
  page.on('console', msg => console.log('PAGE LOG:', msg.text()));
  page.on('pageerror', err => console.log('PAGE ERROR:', err.toString()));
  
  await page.setViewport({ width: 1200, height: 800 });
  
  try {
      console.log("Navigating to http://localhost:3000");
      await page.goto('http://localhost:3000', { waitUntil: 'networkidle0' });
      await new Promise(r => setTimeout(r, 1000));
      
      console.log("Testing Ctrl+E (Edit)");
      await page.keyboard.down('Control');
      await page.keyboard.press('E');
      await page.keyboard.up('Control');
      await new Promise(r => setTimeout(r, 1000));
      
      let editor = await page.$('.editor-container');
      if (!editor) { editor = await page.$('.code-editor-container'); }
      if (!editor) { editor = await page.$('.cm-editor'); }
      
      if (editor) {
          console.log("Editor opened successfully via Ctrl+E.");
          await page.screenshot({ path: 'test_key_edit.png' });
          
          console.log("Testing Escape (Cancel)");
          await page.keyboard.press('Escape');
          await new Promise(r => setTimeout(r, 1000));
          
          editor = await page.$('.cm-editor');
          if (!editor) {
              console.log("Editor closed successfully via Escape.");
          } else {
              console.error("Editor did not close via Escape.");
          }
      } else {
          console.error("Editor did not open via Ctrl+E.");
      }

      console.log("Testing Ctrl+B (Drawer Toggle)");
      await page.keyboard.down('Control');
      await page.keyboard.press('B');
      await page.keyboard.up('Control');
      await new Promise(r => setTimeout(r, 1000));
      
      let drawer = await page.$('.drawer.open');
      if (drawer) {
          console.log("Drawer opened successfully via Ctrl+B.");
          await page.screenshot({ path: 'test_key_drawer.png' });
      } else {
          console.error("Drawer did not open via Ctrl+B.");
      }

      console.log("Testing Ctrl+K (Search)");
      await page.keyboard.down('Control');
      await page.keyboard.press('K');
      await page.keyboard.up('Control');
      await new Promise(r => setTimeout(r, 1000));
      
      let commandPalette = await page.$('.command-palette-overlay');
      if (commandPalette) {
          console.log("Command Palette opened successfully via Ctrl+K.");
          await page.screenshot({ path: 'test_key_search.png' });
      } else {
          console.error("Command Palette did not open via Ctrl+K.");
      }

  } catch(e) {
      console.error(e);
  }
  
  await browser.close();
})();
