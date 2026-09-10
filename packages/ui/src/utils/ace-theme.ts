import 'ace-builds/esm-resolver'

import cssText from '@lumen/assets/styles/ace.css?raw'
import ace from 'ace-builds'

ace['define'](
	'ace/theme/Lumen',
	['require', 'exports', 'module', 'ace/lib/dom'],
	function (require, exports, _module) {
		exports.isDark = false
		exports.cssClass = 'ace-Lumen'
		exports.cssText = cssText

		const dom = require('ace/lib/dom')
		dom.importCssString(exports.cssText, exports.cssClass, false)
	},
)
