token {
  color.surface = #ffffff;
  color.accent = #0A84FF;
  color.text = #2f3542;
  radius.md = 12dp;
  space.md = 12dp;
  text.h1 = 20px;
  text.body = 16px;
}

style HeaderFrame {
  background: token(color.surface);
  border: 1px solid #d4d8e5;
  border-radius: token(radius.md);
  padding: token(space.md);
}

style Heading {
  color: token(color.accent);
  font-size: token(text.h1);
  font-weight: 700;
}

style BodyText {
  color: token(color.text);
  font-size: token(text.body);
}

style Button {
  background: #e8f0ff;
  border: 1px solid #8fb0ff;
}

style Input {
  border: 1px solid #c5cde4;
  border-radius: 8dp;
  padding: 8dp;
}
