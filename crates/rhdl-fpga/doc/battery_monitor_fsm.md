

<p>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 170 880" font-family="sans-serif" font-size="13">
<defs><marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="#444"/></marker></defs>
<title>FSM diagram for BatteryMonitor</title>
<line x1="85" y1="70" x2="85" y2="160" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="85" y1="200" x2="85" y2="290" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="85" y1="330" x2="85" y2="420" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="85" y1="460" x2="85" y2="550" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="85" y1="590" x2="85" y2="680" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="85" y1="720" x2="85" y2="810" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="85" y1="850" x2="85" y2="30" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<rect x="30" y="30" width="110" height="40" rx="6" ry="6" fill="#e0f2ff" stroke="#2563eb" stroke-width="1"/>
<text x="85" y="55" text-anchor="middle">wait</text>
<rect x="30" y="160" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="85" y="185" text-anchor="middle">issue Break</text>
<rect x="30" y="290" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="85" y="315" text-anchor="middle">wait Break</text>
<rect x="30" y="420" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="85" y="445" text-anchor="middle">issue Addr</text>
<rect x="30" y="550" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="85" y="575" text-anchor="middle">wait Addr</text>
<rect x="30" y="680" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="85" y="705" text-anchor="middle">issue Read</text>
<rect x="30" y="810" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="85" y="835" text-anchor="middle">wait Read</text>
</svg>

</p>
