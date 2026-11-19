import styles from './PdfEmbed.module.css';

type Props = {
  /** Path under `public/`, without a leading slash (e.g. `pdfs/mediancut.pdf`). */
  src: string;
  title: string;
  height?: string;
};

export default function PdfEmbed({ src, title, height = 'min(85vh, 900px)' }: Props) {
  const href = import.meta.env.BASE_URL + src.replace(/^\.?\//, '');

  return (
    <figure className={styles.embed}>
      <object
        data={href}
        type="application/pdf"
        title={title}
        aria-label={title}
        style={{ height }}
      >
        <p>
          Your browser cannot display PDFs inline. <a href={href}>Download “{title}”</a>.
        </p>
      </object>
      <figcaption>
        {title} — <a href={href}>open in a new tab</a>
      </figcaption>
    </figure>
  );
}
