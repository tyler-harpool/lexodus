// Court Order PDF Template
// Data is injected via #let bindings at the top of the generated document

#set page(
  paper: "us-letter",
  margin: (top: 1.25in, bottom: 1in, x: 1in),
  header: align(center)[
    #set text(8pt, fill: luma(150))
    #court_name
  ],
  footer: context [
    #set text(8pt, fill: luma(150))
    #align(center)[Page #counter(page).display() of #counter(page).final().first()]
  ],
)

#set text(font: "New Computer Modern", size: 12pt)
#set par(justify: true, leading: 0.65em)

// Header
#align(center)[
  #block(spacing: 0.5em)[
    #text(14pt, weight: "bold")[UNITED STATES DISTRICT COURT]
  ]
  #block(spacing: 0.4em)[
    #text(12pt, weight: "bold")[#upper(court_name)]
  ]
  #v(0.3em)
  #line(length: 100%, stroke: 0.5pt)
  #v(0.5em)
]

// Case info block
#block(inset: (left: 0pt, right: 0pt))[
  #grid(
    columns: (1fr, auto),
    align: (left, right),
    [*Case ID:* #case_id],
    [*Date:* #order_date],
  )
]

#v(0.5em)
#line(length: 100%, stroke: 0.3pt)
#v(0.5em)

// Order type and title
#align(center)[
  #text(13pt, weight: "bold")[#upper(order_type)]
  #v(0.3em)
  #text(12pt, style: "italic")[#title]
]

#v(1em)

// Order content body
#block[
  #content_body
]

// Signature block (conditionally shown)
#if show_signature [
  #v(2em)
  #line(length: 100%, stroke: 0.5pt)
  #v(0.8em)

  #grid(
    columns: (1fr,),
    row-gutter: 0.4em,
    [*Electronically Signed By:* #signer_name],
    [*Date:* #signed_date],
    [*Court:* #court_name],
  )

  #v(0.5em)
  #text(9pt, fill: luma(100))[
    This document has been electronically signed in accordance with
    the Federal Rules of Civil Procedure and the local rules of this court.
  ]
]

// Status footer
#v(1em)
#align(center)[
  #text(9pt, fill: luma(120))[
    Status: #status #h(2em) Order Type: #order_type
  ]
]
