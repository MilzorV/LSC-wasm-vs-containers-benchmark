.PHONY: all clean speaker-notes speaker-notes-clean

TEX := main.tex
PDF := main.pdf

SPEAKER_MD := speaker-plan.md speaker-notes-milosz.md speaker-notes-damian.md
SPEAKER_PDF := speaker-plan.pdf speaker-notes-milosz.pdf speaker-notes-damian.pdf

PANDOC := pandoc
PANDOC_PDF_FLAGS := --pdf-engine=xelatex \
	-V geometry:margin=2.5cm \
	-V fontsize=11pt \
	-V mainfont="Helvetica" \
	-V monofont="Menlo"

all: $(PDF)

$(PDF): $(TEX)
	latexmk -pdf -interaction=nonstopmode -halt-on-error $(TEX)

speaker-notes: $(SPEAKER_PDF)

speaker-plan.pdf: speaker-plan.md
	$(PANDOC) $< -o $@ $(PANDOC_PDF_FLAGS)

speaker-notes-milosz.pdf: speaker-notes-milosz.md
	$(PANDOC) $< -o $@ $(PANDOC_PDF_FLAGS) --toc --toc-depth=2

speaker-notes-damian.pdf: speaker-notes-damian.md
	$(PANDOC) $< -o $@ $(PANDOC_PDF_FLAGS) --toc --toc-depth=2

clean:
	latexmk -C $(TEX)

speaker-notes-clean:
	rm -f $(SPEAKER_PDF)
