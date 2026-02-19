// Civil Judgment Template (FRCP Rule 58)
// Uses the same DocumentParams as document.typ (court_name, case_id, content_body, etc.)

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

#align(center)[
  #text(14pt, weight: "bold")[UNITED STATES DISTRICT COURT]
  #v(0.2em)
  #text(12pt, weight: "bold")[#upper(court_name)]
  #v(0.3em)
  #line(length: 100%, stroke: 0.5pt)
]

#v(0.5em)

#grid(
  columns: (1fr, auto),
  [*Case No.:* #case_id],
  [*Date:* #document_date],
)

#v(0.5em)
#line(length: 100%, stroke: 0.3pt)
#v(0.5em)

#align(center)[
  #text(13pt, weight: "bold")[JUDGMENT IN A CIVIL CASE]
]

#v(1em)

#block[#content_body]

#if show_signature [
  #v(2em)
  #line(length: 100%, stroke: 0.5pt)
  #v(0.8em)
  #grid(
    columns: (1fr,),
    row-gutter: 0.4em,
    [*Electronically Signed By:* Judge (ID: #signer_id)],
    [*Date:* #document_date],
    [*Court:* #court_name],
  )
  #v(0.5em)
  #text(9pt, fill: luma(100))[
    This document has been electronically signed in accordance with
    the Federal Rules of Civil Procedure and the local rules of this court.
  ]
]
