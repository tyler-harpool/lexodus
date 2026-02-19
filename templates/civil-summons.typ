// Federal Civil Summons Template
// Variables are injected via #let bindings

#set page(paper: "us-letter", margin: (top: 1in, bottom: 1in, x: 1in))
#set text(font: "New Computer Modern", size: 11pt)
#set par(justify: true, leading: 0.65em)

#align(center)[
  #text(14pt, weight: "bold")[UNITED STATES DISTRICT COURT]
  #v(0.2em)
  #text(12pt, weight: "bold")[#upper(court_name)]
  #v(0.3em)
  #line(length: 100%, stroke: 0.5pt)
]

#v(0.5em)

// Case caption
#grid(
  columns: (1fr, auto),
  [
    #plaintiff_name, \
    #h(2em) _Plaintiff,_ \
    #v(0.5em)
    #h(1em) v. \
    #v(0.5em)
    #defendant_name, \
    #h(2em) _Defendant._
  ],
  align(right)[
    *Case No.* #case_number \
    #v(0.5em)
    *SUMMONS IN A CIVIL ACTION*
  ],
)

#v(0.5em)
#line(length: 100%, stroke: 0.5pt)
#v(1em)

#text(12pt, weight: "bold")[SUMMONS IN A CIVIL ACTION]

#v(1em)

To: #defendant_name

A lawsuit has been filed against you.

Within 21 days after service of this summons on you (not counting the day you received it) --- or 60 days if you are the United States or a United States agency, or an officer or employee of the United States described in Fed. R. Civ. P. 12(a)(2) or (3) --- you must serve on the plaintiff an answer to the attached complaint or a motion under Rule 12 of the Federal Rules of Civil Procedure. The answer or motion must be served on the plaintiff or plaintiff's attorney, whose name and address are:

#v(0.5em)
#block(inset: (left: 2em, right: 2em))[#attorney_info]
#v(0.5em)

If you fail to respond, judgment by default will be entered against you for the relief demanded in the complaint. You also must file your answer or motion with the court.

#v(2em)

#grid(
  columns: (1fr, 1fr),
  row-gutter: 0.5em,
  [*CLERK OF COURT*], [],
  [*Date:* #document_date], [Signature of Clerk or Deputy Clerk: ________________________],
)
