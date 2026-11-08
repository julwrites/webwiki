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
      await page.keyboard.press('KeyE');
      await page.keyboard.up('Control');
      await new Promise(r => setTimeout(r, 1000));
      
      let editor = await page.$('.cm-editor');
      if (editor) {
          console.log("✅ Editor opened successfully via Ctrl+E.");
          await page.screenshot({ path: 'test_key_edit.png' });

          // Test Vim Normal mode Undo ('u') and Redo ('r' and '<Ctrl+r>')
          console.log("Testing Vim Insert + Undo ('u') + Redo ('r' and '<Ctrl+r>')...");
          await page.keyboard.press('a');
          await page.keyboard.type('TEST_UNDO_REDO');
          await page.keyboard.press('Escape');
          await new Promise(r => setTimeout(r, 300));

          let contentAfterInsert = await page.evaluate(() => window.getEditorContent('code-editor'));
          console.log("Content includes TEST_UNDO_REDO after insert:", contentAfterInsert.includes('TEST_UNDO_REDO'));

          // Press 'u' in Normal mode to undo
          await page.keyboard.press('u');
          await new Promise(r => setTimeout(r, 300));
          let contentAfterUndo = await page.evaluate(() => window.getEditorContent('code-editor'));
          if (!contentAfterUndo.includes('TEST_UNDO_REDO')) {
              console.log("✅ Undo ('u') in Vim Normal mode succeeded.");
          } else {
              console.error("❌ Undo ('u') did not remove inserted text.");
          }

          // Test '<Ctrl+r>' redo in Normal mode
          await page.keyboard.down('Control');
          await page.keyboard.press('KeyR');
          await page.keyboard.up('Control');
          await new Promise(r => setTimeout(r, 300));
          let contentAfterRedoCtrlR = await page.evaluate(() => window.getEditorContent('code-editor'));
          if (contentAfterRedoCtrlR.includes('TEST_UNDO_REDO')) {
              console.log("✅ Redo ('<Ctrl+r>') in Vim Normal mode succeeded.");
          } else {
              console.error("❌ Redo ('<Ctrl+r>') did not restore inserted text.");
          }

          // Test 'r' in Normal mode performs replace character (e.g. 'r' then 'X')
          await page.keyboard.press('u');
          await new Promise(r => setTimeout(r, 300));
          await page.keyboard.press('r');
          await page.keyboard.press('X');
          await new Promise(r => setTimeout(r, 300));
          let contentAfterReplace = await page.evaluate(() => window.getEditorContent('code-editor'));
          if (contentAfterReplace.includes('X')) {
              console.log("✅ Replace ('r<char>') in Vim Normal mode succeeded.");
          } else {
              console.error("❌ Replace ('r<char>') did not replace character.");
          }

          // Undo cleanup before testing <Ctrl+p><Ctrl+f> in Normal mode
          await page.keyboard.press('u');
          await new Promise(r => setTimeout(r, 200));

          // Test <Ctrl+p><Ctrl+f> in Vim Normal mode inside the editor
          console.log("Testing <Ctrl+p><Ctrl+f> in Vim Normal mode...");
          await page.keyboard.down('Control');
          await page.keyboard.press('KeyP');
          await page.keyboard.press('KeyF');
          await page.keyboard.up('Control');
          await new Promise(r => setTimeout(r, 500));

          let paletteInNormal = await page.$('.command-palette-overlay');
          if (paletteInNormal) {
              console.log("✅ Command Palette opened via <Ctrl+p><Ctrl+f> in Vim Normal mode.");
              await page.keyboard.press('Escape');
              await new Promise(r => setTimeout(r, 300));
          } else {
              console.error("❌ Command Palette did NOT open via <Ctrl+p><Ctrl+f> in Vim Normal mode.");
          }

          // Verify editor is still open after closing Command Palette with Escape!
          editor = await page.$('.cm-editor');
          if (editor) {
              console.log("✅ Editor remained open and focused after closing Command Palette.");
          } else {
              console.error("❌ Editor unexpectedly closed when dismissing Command Palette.");
          }

          // Exit editor via Cancel button
          const cancelBtn = await page.$('button[aria-label="Cancel editing"]');
          if (cancelBtn) {
              await cancelBtn.click();
              await new Promise(r => setTimeout(r, 500));
          }
      } else {
          console.error("❌ Editor did not open via Ctrl+E.");
      }

      console.log("Testing Ctrl+B (Drawer Toggle)");
      await page.keyboard.down('Control');
      await page.keyboard.press('KeyB');
      await page.keyboard.up('Control');
      await new Promise(r => setTimeout(r, 1000));
      
      let drawer = await page.$('.drawer.open');
      if (drawer) {
          console.log("✅ Drawer opened successfully via Ctrl+B.");
          await page.screenshot({ path: 'test_key_drawer.png' });
          // Close drawer
          await page.keyboard.down('Control');
          await page.keyboard.press('KeyB');
          await page.keyboard.up('Control');
          await new Promise(r => setTimeout(r, 500));
      } else {
          console.error("❌ Drawer did not open via Ctrl+B.");
      }

      console.log("Testing <Ctrl+p><Ctrl+f> (Global Search / Open Document)");
      await page.keyboard.down('Control');
      await page.keyboard.press('KeyP');
      await page.keyboard.press('KeyF');
      await page.keyboard.up('Control');
      await new Promise(r => setTimeout(r, 1000));
      
      let commandPalette = await page.$('.command-palette-overlay');
      if (commandPalette) {
          console.log("✅ Command Palette opened successfully via <Ctrl+p><Ctrl+f>.");
          await page.screenshot({ path: 'test_key_search.png' });
      } else {
          console.error("❌ Command Palette did not open via <Ctrl+p><Ctrl+f>.");
      }

  } catch(e) {
      console.error(e);
  }
  
  await browser.close();
})();
