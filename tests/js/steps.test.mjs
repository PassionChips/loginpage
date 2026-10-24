// Unit tests for the step icons: each step's icon comes from what the step asks you to do.
import test from 'node:test';
import assert from 'node:assert/strict';
import { actionOf } from '../../static/js/guide/steps.mjs';

test('a step gets the icon of its action, from its own words', () => {
  assert.equal(actionOf('Open Sarathi and choose your state (for example Tamil Nadu).'), 'open');
  assert.equal(actionOf('Pay the fee online. After paying, wait for the message.'), 'pay');
  assert.equal(actionOf('Upload your photograph and signature.'), 'upload');
  assert.equal(actionOf('LL slot book: choose a date and time for the test.'), 'book');
  assert.equal(actionOf('Take the LL test. Sarathi also offers an online test.'), 'test');
  assert.equal(actionOf('Save or print your application as a PDF.'), 'print');
  assert.equal(actionOf('Check every detail and click SUBMIT.'), 'verify');
  assert.equal(actionOf('Fill in every field. Mobile number and e-mail are mandatory.'), 'fill');
  assert.equal(actionOf('Hand your documents to the principal within 7 days of admission.'), 'step', 'no keyword: a numbered node');
});
