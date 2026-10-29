<!-- SPDX-License-Identifier: MIT -->
<!--
	The refraction the toolbar's glass bends its backdrop with (`.lrail`,
	`surfaces.css`), mounted once by the shell. Low-frequency noise, blurred
	smooth, used as a displacement map: gentle, lens-like warping. The fixed
	seed draws it the same on every render.
-->
<svg width="0" height="0" aria-hidden="true" style="position: absolute">
	<filter id="lg-refract" x="0" y="0" width="100%" height="100%" color-interpolation-filters="sRGB">
		<feTurbulence type="fractalNoise" baseFrequency="0.011 0.006" numOctaves="2" seed="11" result="noise" />
		<feGaussianBlur in="noise" stdDeviation="2.5" result="soft" />
		<feDisplacementMap in="SourceGraphic" in2="soft" scale="34" xChannelSelector="R" yChannelSelector="G" />
	</filter>
</svg>
