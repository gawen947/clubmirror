// clubmirror default theme: dates, modals and sharing. No dependencies.
(function () {
  'use strict';

  var tz = document.documentElement.dataset.timezone || undefined;
  var lastFocused = null;

  // Dates are rendered on the server in the club timezone (works without JS).
  // Here we reformat them with the visitor's language, still in the club timezone.
  function localizeDates() {
    document.querySelectorAll('time[datetime]').forEach(function (el) {
      var date = new Date(el.getAttribute('datetime'));
      if (isNaN(date.getTime())) return;
      try {
        el.textContent = new Intl.DateTimeFormat(undefined, {
          weekday: 'long', year: 'numeric', month: 'long', day: 'numeric',
          hour: '2-digit', minute: '2-digit', timeZone: tz
        }).format(date);
      } catch (e) { /* unknown timezone or old browser: keep the server version */ }
    });
  }

  function openModal(id) {
    var modal = document.getElementById(id);
    if (!modal) return;
    lastFocused = document.activeElement;
    modal.classList.add('show');
    document.body.style.overflow = 'hidden';
    var close = modal.querySelector('[data-close]');
    if (close) close.focus();
  }

  function closeModal(modal) {
    modal.classList.remove('show');
    document.body.style.overflow = '';
    if (lastFocused) lastFocused.focus();
  }

  function showToast() {
    var toast = document.getElementById('share-toast');
    if (!toast) return;
    toast.textContent = toast.dataset.text || '';
    toast.classList.add('show');
    setTimeout(function () { toast.classList.remove('show'); }, 3000);
  }

  function shareEvent(id) {
    var url = new URL(window.location);
    url.search = '';
    url.hash = '';
    url.searchParams.set('event', id);
    var text = url.toString();

    function fallback() {
      var area = document.createElement('textarea');
      area.value = text;
      document.body.appendChild(area);
      area.select();
      document.execCommand('copy');
      document.body.removeChild(area);
      showToast();
    }

    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(showToast, fallback);
    } else {
      fallback();
    }
  }

  document.addEventListener('click', function (event) {
    var target = event.target;
    var open = target.closest('[data-open]');
    if (open) return openModal(open.dataset.open);
    var share = target.closest('[data-share]');
    if (share) return shareEvent(share.dataset.share);
    if (target.closest('[data-close]')) return closeModal(target.closest('.modal'));
    if (target.classList.contains('modal')) closeModal(target); // click on the backdrop
  });

  document.addEventListener('keydown', function (event) {
    if (event.key !== 'Escape') return;
    var modal = document.querySelector('.modal.show');
    if (modal) closeModal(modal);
  });

  document.addEventListener('DOMContentLoaded', function () {
    localizeDates();
    // shared link: ?event=<id> opens the matching event
    var id = new URLSearchParams(window.location.search).get('event');
    if (id) openModal('modal-' + id);
  });
})();
