const tracker = document.querySelector('script[data-website-id]');
const website = tracker?.dataset.websiteId;

window.addEventListener('load', () => {
  if (window.umami && website) {
    window.umami.track({ website, url: '/website', title: 'Rawrarr website' });
  }
});
