# Changelog

## 0.1.0 (2026-11-08)


### Features

* add default template ([14957c6](https://github.com/gawen947/clubmirror/commit/14957c67738dcb939f1edc68881100862e10e13a))
* add dependencies ([6bbd3a4](https://github.com/gawen947/clubmirror/commit/6bbd3a48569c1e939495bf871b70bf6844971b22))
* **calendar:** add calendar name + timezone + stable UID ([d696551](https://github.com/gawen947/clubmirror/commit/d6965518c1e4e0a2081cc58df35d0144f21c94ec))
* **calendar:** add generation of a calendar ([6034308](https://github.com/gawen947/clubmirror/commit/60343081dd4470bab09964dae6beea3bd1181fcf))
* **config:** remap config sections ([22a43e3](https://github.com/gawen947/clubmirror/commit/22a43e360e50e586cadd03da503e628260c23c98))
* **config:** rename + add --config and no longer require root ([976a039](https://github.com/gawen947/clubmirror/commit/976a0396cc56e043671c390bf3ccb6ced26e37a5))
* local JSON mode + site context ([1569a45](https://github.com/gawen947/clubmirror/commit/1569a451d03efcbca3ef9d153ebc78663fe70712))
* **main:** add quirk save json ([95f3fcd](https://github.com/gawen947/clubmirror/commit/95f3fcd525e9ae47ac51e293cfb9b50490be8551))
* **main:** add quirk use local json ([b2f01d1](https://github.com/gawen947/clubmirror/commit/b2f01d1e95ef549324b1df1f9292681c73784c9b))
* **main:** add sync events fetching ([0ccc125](https://github.com/gawen947/clubmirror/commit/0ccc125ebf7da644afeaea41b6ea30c12f58b8bb))
* **main:** apply bkn early ride quirk if needed ([50948fa](https://github.com/gawen947/clubmirror/commit/50948fa0d00c6d01909cdbb4e39d6739afbe4643))
* **main:** filter events ([cd89e2b](https://github.com/gawen947/clubmirror/commit/cd89e2b6ce6d22cf142d77f3a6b5cf8482dae545))
* **main:** generate the template ([f97200b](https://github.com/gawen947/clubmirror/commit/f97200b443f2a60dbcf1f576ec6566aa492ddc6d))
* **main:** put config within directory ([6e8b44c](https://github.com/gawen947/clubmirror/commit/6e8b44c59369b2e9ad48f6d09b49d8d46c93dafb))
* optimize release ([7522ef3](https://github.com/gawen947/clubmirror/commit/7522ef301aa6b508cb3efa35f68d5e49ec80aaee))
* **quirk:** add a quirk to reschedule BKN early rides ([39dde4f](https://github.com/gawen947/clubmirror/commit/39dde4fafd1dd35626949fda8422d4d97984e490))
* **quirk:** add optional boundary for bkn_early_ride ([2272595](https://github.com/gawen947/clubmirror/commit/2272595d5d0b0a2bf4c769bf4313b6875ae4675d))
* **quirk:** add quirk_sort_events ([9468844](https://github.com/gawen947/clubmirror/commit/946884465a22c256321a0b74b830f0ac959d45ee))
* **quirk:** remove club specific quirks ([3200572](https://github.com/gawen947/clubmirror/commit/3200572873ff7172e2972fede38efa23841327ad))
* **routes:** handle downloading routes ([fe87178](https://github.com/gawen947/clubmirror/commit/fe871786ef30bf72bd172360241dd5e09ee1a0b2))
* **routes:** make downloading routes optional ([f0d12d1](https://github.com/gawen947/clubmirror/commit/f0d12d1712cc73393e0d8f5793231e2fb244d6cb))
* **strava:** adapt to optional types ([4d8eb21](https://github.com/gawen947/clubmirror/commit/4d8eb21286eb18be430a917dea0d41aefdf7c0ce))
* **strava:** add strava API ([edc902f](https://github.com/gawen947/clubmirror/commit/edc902fab3167ee5eca33f4180a4e8f143be9265))
* **strava:** allow to only warn on 404 ([f2a2057](https://github.com/gawen947/clubmirror/commit/f2a2057360c6ed2f9fd50117ee45f58a2a176298))
* **strava:** make athlete details optional ([8987232](https://github.com/gawen947/clubmirror/commit/898723294d40f76154a26c9284109b67fcbc051b))
* **strava:** persit rotated refresh token ([ca3a772](https://github.com/gawen947/clubmirror/commit/ca3a77277db32ca046e1ab8c91297cddbfa9d297))
* **strava:** save strava response for debug ([9cb3aec](https://github.com/gawen947/clubmirror/commit/9cb3aec3a5c2ceecd15123fe684619d7d9550fb1))
* **template:** add code to generate templates ([c10a377](https://github.com/gawen947/clubmirror/commit/c10a3778a626c6a521c625a8b41f1930f935ec07))
* **template:** add routes_url ([0dd70f6](https://github.com/gawen947/clubmirror/commit/0dd70f645ccff5a8798835dc943ec0a36a02420f))
* **template:** pass club ID to context ([79ca08b](https://github.com/gawen947/clubmirror/commit/79ca08bcc2c28258242f437bee0309f08b4109ce))
* **template:** render only files with a template extension, copy the rest ([cd654f6](https://github.com/gawen947/clubmirror/commit/cd654f62e6faa58ea23402c3238abcd2165a78e3))
* **template:** report missing template directories ([e711f87](https://github.com/gawen947/clubmirror/commit/e711f87b8d7cd8440fc559517a182f5b8f587473))
* update to tera 2. ([c39f3fe](https://github.com/gawen947/clubmirror/commit/c39f3fe279354c8a1f063a625d5f77b1b0d2592a))
* **utils:** add a method a tests for getting the next tuesday at 6AM ([ab4a3c7](https://github.com/gawen947/clubmirror/commit/ab4a3c7f4f6a837755b6db477707085786241793))
* **utils:** add function to get user/group ([1f475f5](https://github.com/gawen947/clubmirror/commit/1f475f54465b8f9557459b72c111af43af140c21))


### Bug Fixes

* **main:** correctly parse uid/gid for generated templates ([0b2ade9](https://github.com/gawen947/clubmirror/commit/0b2ade9682123189c6ecbffb2e7a57e7501006d5))
* **main:** return new access_token when refreshed ([6fcb36e](https://github.com/gawen947/clubmirror/commit/6fcb36e1d6844680a859e1f15a897d483492adef))
* rename sample config ([03fc46a](https://github.com/gawen947/clubmirror/commit/03fc46a383933081c26ba9a0fb6e4b9044623570))
* **routes:** use API to download routes ([0a57f4d](https://github.com/gawen947/clubmirror/commit/0a57f4d8aa6ac80c90f83eef500044741e7e9601))
* **strava:** do not fail on unknown status code ([b511a06](https://github.com/gawen947/clubmirror/commit/b511a064d28085d4a62a8ae32de3fa43c6f9c824))
* **utils:** correctly import and report errors ([0f69eb6](https://github.com/gawen947/clubmirror/commit/0f69eb623c199c04c3e3f26947b347050595726c))
