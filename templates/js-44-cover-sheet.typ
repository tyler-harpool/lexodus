// JS-44 Civil Cover Sheet Template
// Variables are injected via #let bindings

#set page(paper: "us-letter", margin: (top: 0.75in, bottom: 0.75in, x: 0.75in))
#set text(font: "New Computer Modern", size: 10pt)

// Title block
#align(center)[
  #text(12pt, weight: "bold")[JS 44 (Rev. 04/21)]
  #v(0.3em)
  #text(14pt, weight: "bold")[CIVIL COVER SHEET]
  #v(0.2em)
  #text(9pt)[The JS 44 civil cover sheet and the information contained herein neither replace nor supplement the filing and service of pleadings or other papers as required by law.]
]

#v(0.5em)
#line(length: 100%, stroke: 1pt)
#v(0.3em)

// I. Parties
#text(weight: "bold")[I. (a) PLAINTIFFS]
#v(0.2em)
#block(inset: (left: 1em))[#plaintiff_name]
#v(0.3em)
#text(weight: "bold")[(b) DEFENDANTS]
#v(0.2em)
#block(inset: (left: 1em))[#defendant_name]

#v(0.3em)
#text(weight: "bold")[(c) County of Residence: ] #county
#v(0.3em)
#text(weight: "bold")[(d) Attorneys (Firm Name, Address, and Telephone Number)]
#block(inset: (left: 1em))[#attorney_info]

#v(0.5em)
#line(length: 100%, stroke: 0.5pt)
#v(0.3em)

// II. Basis of Jurisdiction
#text(weight: "bold")[II. BASIS OF JURISDICTION] #h(1em) #text(style: "italic")[(Place an "X" in One Box Only)]
#v(0.2em)
#grid(
  columns: (1fr, 1fr),
  row-gutter: 0.3em,
  [#if jurisdiction_basis == "federal_question" [X] else [\_] U.S. Government Plaintiff / Federal Question],
  [#if jurisdiction_basis == "diversity" [X] else [\_] Diversity],
  [#if jurisdiction_basis == "us_government_plaintiff" [X] else [\_] U.S. Government Plaintiff],
  [#if jurisdiction_basis == "us_government_defendant" [X] else [\_] U.S. Government Defendant],
)

#v(0.5em)
#line(length: 100%, stroke: 0.5pt)
#v(0.3em)

// III. Citizenship of Parties (for diversity cases)
#text(weight: "bold")[III. CITIZENSHIP OF PRINCIPAL PARTIES]
#v(0.2em)
#text(9pt, style: "italic")[(For Diversity Cases Only)]

#v(0.5em)
#line(length: 100%, stroke: 0.5pt)
#v(0.3em)

// IV. Nature of Suit
#text(weight: "bold")[IV. NATURE OF SUIT] #h(1em) #text(style: "italic")[(Place an "X" in One Box Only)]
#v(0.2em)
#text[NOS Code: *#nature_of_suit* --- #nos_description]

#v(0.5em)
#line(length: 100%, stroke: 0.5pt)
#v(0.3em)

// V. Origin
#text(weight: "bold")[V. ORIGIN] #h(1em) #text(style: "italic")[(Place an "X" in One Box Only)]
#v(0.2em)
#text[X 1. Original Proceeding]

#v(0.5em)
#line(length: 100%, stroke: 0.5pt)
#v(0.3em)

// VI. Cause of Action
#text(weight: "bold")[VI. CAUSE OF ACTION]
#v(0.2em)
#block(inset: (left: 1em))[#cause_of_action]

#v(0.5em)
#line(length: 100%, stroke: 0.5pt)
#v(0.3em)

// VII. Requested in Complaint
#text(weight: "bold")[VII. REQUESTED IN COMPLAINT]
#v(0.2em)
#grid(
  columns: (auto, auto, auto),
  column-gutter: 2em,
  [#if class_action [X] else [\_] CLASS ACTION],
  [JURY DEMAND: #jury_demand],
  [#if amount_in_controversy != "" [DEMAND: \$#amount_in_controversy] else [DEMAND: N/A]],
)

#v(0.5em)
#line(length: 100%, stroke: 0.5pt)
#v(0.3em)

// VIII. Related Cases
#text(weight: "bold")[VIII. RELATED CASE(S) IF ANY]
#v(0.2em)
#text[Case Number: #case_number]

#v(1em)
#grid(
  columns: (1fr, 1fr),
  [*Date:* #document_date],
  [*Signature of Attorney of Record:* ________________________],
)
