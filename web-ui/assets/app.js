/* course-ui — the only client-side script.
 *
 * Everything that matters works without it: pages are server-rendered, the
 * completion form is a plain POST, and the theme falls back to the reader's
 * system preference. This file adds the two things a server round-trip makes
 * genuinely worse — flipping the theme, and copying a snippet.
 *
 * The blocking part (reading the stored theme before first paint, so the page
 * does not flash) lives inline in <head>; see web-ui/src/page.rs. */

(function () {
  'use strict';

  var STORAGE_KEY = 'course-ui-theme';

  /* ---- theme toggle ---------------------------------------------------- */

  function systemTheme() {
    return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
  }

  function currentTheme() {
    return document.documentElement.getAttribute('data-theme') || systemTheme();
  }

  function applyTheme(theme) {
    document.documentElement.setAttribute('data-theme', theme);
    try {
      localStorage.setItem(STORAGE_KEY, theme);
    } catch (err) {
      /* Private mode or storage disabled: the theme still applies for this
         page view, it just will not be remembered. Not worth reporting. */
    }
  }

  var toggle = document.querySelector('.theme-toggle');
  if (toggle) {
    toggle.addEventListener('click', function () {
      applyTheme(currentTheme() === 'dark' ? 'light' : 'dark');
    });
  }

  /* ---- copy buttons on code blocks ------------------------------------- */

  var copyLabel = document.documentElement.lang === 'fa' ? 'کپی' : 'Copy';
  var copiedLabel = document.documentElement.lang === 'fa' ? 'کپی شد' : 'Copied';

  document.querySelectorAll('.reading pre > code').forEach(function (code) {
    var pre = code.parentElement;
    var button = document.createElement('button');
    button.type = 'button';
    button.className = 'copy-code';
    button.textContent = copyLabel;

    button.addEventListener('click', function () {
      var write = navigator.clipboard && navigator.clipboard.writeText
        ? navigator.clipboard.writeText(code.textContent)
        : Promise.reject(new Error('clipboard unavailable'));

      write.then(function () {
        button.textContent = copiedLabel;
        setTimeout(function () { button.textContent = copyLabel; }, 1600);
      }, function () {
        /* Insecure origin or a denied permission — say so rather than
           pretending it worked. */
        button.textContent = '✕';
        setTimeout(function () { button.textContent = copyLabel; }, 1600);
      });
    });

    pre.appendChild(button);
  });

  /* ---- self-check controls --------------------------------------------- */

  /* Each control is a real <form> that posts and redirects back; the script
     only submits it on change so the reader does not have to hunt for a save
     button. With JS off, the forms still have their own submit buttons. */
  document.querySelectorAll('form[data-autosubmit]').forEach(function (form) {
    form.querySelectorAll('input[type=checkbox]').forEach(function (input) {
      input.addEventListener('change', function () { form.requestSubmit(); });
    });
  });
})();
