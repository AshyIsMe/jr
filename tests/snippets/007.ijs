double_greater =: {{
  hack =: x
  if. x > y do.
    b =: x
  else.
    b =: y
  end.
  b * 2
}}
(6 double_greater 5); (4 double_greater 3)
