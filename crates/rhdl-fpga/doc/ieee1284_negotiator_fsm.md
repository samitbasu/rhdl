

<p>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 590 1010" font-family="sans-serif" font-size="13">
<defs><marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="#444"/></marker></defs>
<title>FSM diagram for Ieee1284Negotiator</title>
<line x1="295" y1="70" x2="295" y2="160" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="295" y1="200" x2="295" y2="290" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="295" y1="330" x2="225" y2="420" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="295" y1="330" x2="365" y2="420" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="225" y1="460" x2="225" y2="550" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="225" y1="590" x2="225" y2="680" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="225" y1="590" x2="365" y2="420" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="225" y1="720" x2="85" y2="810" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="225" y1="720" x2="225" y2="810" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="225" y1="720" x2="365" y2="810" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="85" y1="850" x2="225" y2="810" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="225" y1="850" x2="295" y2="940" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="295" y1="980" x2="295" y2="30" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="295" y1="980" x2="365" y2="550" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="365" y1="590" x2="365" y2="680" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="365" y1="720" x2="295" y2="30" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="365" y1="720" x2="505" y2="810" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="365" y1="460" x2="365" y2="550" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="365" y1="850" x2="365" y2="550" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<line x1="505" y1="850" x2="295" y2="30" stroke="#666" stroke-width="1.5" marker-end="url(#arrow)"/>
<rect x="240" y="30" width="110" height="40" rx="6" ry="6" fill="#e0f2ff" stroke="#2563eb" stroke-width="1"/>
<text x="295" y="55" text-anchor="middle">idle</text>
<rect x="240" y="160" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="295" y="185" text-anchor="middle">setup (Event 0)</text>
<rect x="240" y="290" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="295" y="315" text-anchor="middle">wait device ready</text>
<rect x="170" y="420" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="225" y="445" text-anchor="middle">strobe data</text>
<rect x="170" y="550" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="225" y="575" text-anchor="middle">wait device ack</text>
<rect x="170" y="680" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="225" y="705" text-anchor="middle">check mode</text>
<rect x="30" y="810" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="85" y="835" text-anchor="middle">capture ELI</text>
<rect x="170" y="810" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="225" y="835" text-anchor="middle">host ack</text>
<rect x="240" y="940" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="295" y="965" text-anchor="middle">done</text>
<rect x="310" y="550" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="365" y="575" text-anchor="middle">terminate req</text>
<rect x="310" y="680" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="365" y="705" text-anchor="middle">terminate wait</text>
<rect x="310" y="420" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="365" y="445" text-anchor="middle">not compliant</text>
<rect x="310" y="810" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="365" y="835" text-anchor="middle">mode rejected</text>
<rect x="450" y="810" width="110" height="40" rx="6" ry="6" fill="#ffffff" stroke="#444" stroke-width="1"/>
<text x="505" y="835" text-anchor="middle">timeout</text>
</svg>

</p>
