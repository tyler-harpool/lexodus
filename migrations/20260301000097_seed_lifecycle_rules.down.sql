DELETE FROM rules WHERE name IN (
    'Arraignment: Advance Status on Indictment',
    'Discovery: Advance Status on Answer',
    'Sentencing: Advance Status on Verdict',
    'Sentenced: Advance Status and Create Appeal Deadline',
    'Appeal: Advance Status on Notice of Appeal'
);
