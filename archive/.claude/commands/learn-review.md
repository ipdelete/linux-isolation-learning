# Review and Guide

Review a file and ask leading questions to help the user find the answer themselves.

## Usage
`/learn-review <file_path> <question>`

Example:
`/learn-review ./src/somefile.py "How do I ensure both things happen?"`

## Implementation

The command will:
1. Read the specified file
2. Understand your question about it
3. Ask you a series of leading questions to guide your thinking
4. NOT give you the answer directly
5. Help you work through the problem step by step

This is a learning tool - the goal is for YOU to discover the answer by thinking critically about the code.

---

You're about to review a file and get guided learning feedback. Here's what will happen:

1. I'll read the file you specified
2. I'll understand your question
3. Instead of answering directly, I'll ask you leading questions that guide your thinking
4. I might ask about:
   - What the code is currently doing
   - What you expect to happen
   - What assumptions you're making
   - How different parts connect
   - Edge cases or timing issues
5. Through these questions, you'll work toward the answer yourself

This approach helps you develop problem-solving skills and deeper understanding.

**What file and question do you want to explore?** (Provide them as: `/learn-review <file> <question>`)