

<p>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 450 490" font-family="sans-serif" font-size="13">
<defs><marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="#444"/></marker></defs>
<title>FSM diagram for OneWireMaster</title>
<line x1="225" y1="70" x2="85" y2="160" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="225" y1="70" x2="225" y2="160" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="225" y1="70" x2="365" y2="160" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="85" y1="200" x2="85" y2="290" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="85" y1="330" x2="225" y2="420" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="225" y1="200" x2="225" y2="290" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="225" y1="330" x2="225" y2="160" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="225" y1="330" x2="225" y2="420" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="365" y1="200" x2="365" y2="290" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="365" y1="330" x2="365" y2="160" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="365" y1="330" x2="225" y2="420" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="225" y1="460" x2="225" y2="30" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<rect x="170" y="30" width="110" height="40" rx="6" ry="6" fill="#e0f2ff" stroke="#2563eb" stroke-width="1"/>
<text x="225" y="55" text-anchor="middle">idle</text>
<rect x="30" y="160" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="85" y="185" text-anchor="middle">Reset (low)</text>
<rect x="30" y="290" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="85" y="315" text-anchor="middle">Reset (sample)</text>
<rect x="170" y="160" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="225" y="185" text-anchor="middle">Write (low)</text>
<rect x="170" y="290" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="225" y="315" text-anchor="middle">Write (wait)</text>
<rect x="310" y="160" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="365" y="185" text-anchor="middle">Read (low)</text>
<rect x="310" y="290" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="365" y="315" text-anchor="middle">Read (sample)</text>
<rect x="170" y="420" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="225" y="445" text-anchor="middle">stop</text>
</svg>

</p>
