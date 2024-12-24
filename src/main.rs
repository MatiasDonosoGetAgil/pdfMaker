use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use ttf_parser::Face;

pub struct IOrder {
    pub comentario: Option<String>,
    pub items: Vec<Item>,
    pub sub_total: f64,
    pub gastos_envio: f64,
    pub dscto_cupon_gasto_envio: f64,
    pub dscto_cupon_subtotal: f64,
    pub dscto_puntos: f64,
    pub cupones: Option<Vec<Cupon>>,
    pub fechas: Fechas,
    pub tipo_entrega: TipoEntrega,
    pub drop_off: Option<DropOff>,
    pub courier: Courier,
    pub cliente: Cliente,
    pub sucursal: Option<Sucursal>,
    pub plataforma: Plataforma,
    pub correlativo: i64,
    pub codigo: String,
    pub comercio: Comercio,
    pub pago: Pago,
}

/// Representa un producto o ítem dentro de la orden.
pub struct Item {
    pub cantidad: f64,                  // Equivalente a `number` en TS
    pub nombre: String,
    pub precio: f64,
    pub opciones: Option<Vec<IOpciones>>,
    pub comentario: Option<String>,
}

/// Representa una opción o modificador de un ítem (equivalente a IOpciones).
pub struct IOpciones {
    pub modificador: String,
    pub cantidad: i32,
    pub opcion: String,
}

/// Representa un cupón aplicado a la orden.
pub struct Cupon {
    pub codigo: String,
}

/// Fechas relevantes para la orden, como hora de pago, hora de salida de cocina, etc.
pub struct Fechas {
    pub fecha_salida_cocina_estimada: String,
    pub tz: String,
    pub fecha_pago: String,
}

/// Tipo de entrega (por ejemplo, 1 = Delivery, 2 = Retiro en local).
pub struct TipoEntrega {
    pub id: i32,
}

/// Información opcional cuando es envío a domicilio.
pub struct DropOff {
    pub tipo_entrega: Option<String>,
    pub direccion: Option<String>,
}

/// Información sobre el courier (por ejemplo, IdCourier = -2 cuando es propio).
pub struct Courier {
    pub id_courier: i32,
}

/// Datos del cliente.
pub struct Cliente {
    pub telefono: Option<String>,
    pub nombre: Option<String>,
    pub nro: Option<String>,
}

/// Sucursal (en caso de retiro en tienda).
pub struct Sucursal {
    pub nombre: Option<String>,
}

/// Plataforma desde donde llega la orden (AGIL, etc.).
pub struct Plataforma {
    pub codigo: Option<String>,
    pub nombre: Option<String>,
}

/// Datos del comercio o restaurante que recibe la orden.
pub struct Comercio {
    pub nombre: Option<String>,
}

/// Información del pago (medio de pago, etc.).
pub struct Pago {
    pub medio_pago: MedioPago,
}

/// Medio de pago específico (efectivo, tarjeta, etc.).
pub struct MedioPago {
    pub nombre: Option<String>,
}

struct PdfResources<'a> {
    font: IndirectFontRef,
    face: Face<'a>,
    upem: f64,
    page_width: f64,
    page_height: f64,
    doc: PdfDocumentReference,
    page: PdfPageIndex,
    layer: PdfLayerIndex,
}

impl<'a> PdfResources<'a> {
    fn new() -> Self {
        
        let page_width = 80.0;
        let page_height = 190.0;
        let (
            doc,
            page,
            layer)
            = PdfDocument::new(
                "Ticket",
                Mm(
                    page_width as f32
                ), Mm(
                    page_height as f32
                ), "");
        let font_data: &[u8] = include_bytes!("../assets/fonts/Roboto-Black.ttf") as &[u8];
        let face: Face<'_> = Face::parse(font_data, 0).expect("No se pudo cargar la fuente");
        let upem: f64 = face.units_per_em() as f64;
        
        // let font: IndirectFontRef = doc.add_external_font(font_data).unwrap();
        let font: IndirectFontRef = doc.add_external_font(font_data)
    .expect("Failed to add external font. Ensure the font data is correct and file path is valid.");

        PdfResources {
            font,
            face,
            upem,
            page_width,
            page_height,
            doc,
            page,
            layer
        }
    }
}

fn pdf(orden: &IOrder) {
    
    let resources = PdfResources::new();
            
    let current_layer: PdfLayerReference = resources.doc.get_page(resources.page).get_layer(resources.layer);

    // comercio nombre
    texto_centrado(
        &current_layer,
        24.0,
        orden.comercio.nombre.as_ref().unwrap(),
        10.0,
        &resources,
    );

    // plataforma nombre
    texto_centrado(
        &current_layer,
        16.0,
        orden.plataforma.nombre.as_ref().unwrap(),
        15.0,
        &resources,
    );



    let mut buffer: BufWriter<File> = BufWriter::new(File::create("test_working.pdf").unwrap());
    resources.doc.save(&mut buffer).unwrap();
}

fn texto_centrado(
    current_layer: &PdfLayerReference,
    font_size: f64,
    text: &String,
    altura: f64,
    resources: &PdfResources) {

    let scale_factor = font_size / resources.upem;
    
    let text_width_points: f64 = text
        .chars()
        .filter_map(|c| resources.face.glyph_index(c))
        .map(|glyph_id| resources.face.glyph_hor_advance(glyph_id).unwrap_or(0) as f64)
        .sum::<f64>() * scale_factor;

    let text_width_mm: f64 = text_width_points * 0.352778;
    let x_position: f64 = (resources.page_width - text_width_mm) / 2.0;
    let y_position: f64 = resources.page_height - altura;
    println!("{}", text);
    current_layer.use_text(
        text,
        font_size as f32,
        Mm(x_position as f32),
        Mm(y_position as f32),
        &resources.font,
    );
}

fn main() {
    let orden_ejemplo = IOrder {
        comentario: Some("Orden de ejemplo".to_string()),
        items: vec![
            Item {
                cantidad: 2.0,
                nombre: "Pizza Napolitana".to_string(),
                precio: 2500.0,
                opciones: Some(vec![
                    IOpciones {
                        modificador: "Extra queso".to_string(),
                        cantidad: 1,
                        opcion: "Mozzarella".to_string(),
                    },
                    IOpciones {
                        modificador: "Sin cebolla".to_string(),
                        cantidad: 1,
                        opcion: "Quitar cebolla".to_string(),
                    },
                ]),
                comentario: Some("Sin aceitunas, por favor".to_string()),
            },
        ],
        sub_total: 5000.0,
        gastos_envio: 1000.0,
        dscto_cupon_gasto_envio: 0.0,
        dscto_cupon_subtotal: 0.0,
        dscto_puntos: 0.0,
        cupones: None,
        fechas: Fechas {
            fecha_salida_cocina_estimada: "2024-12-25T14:30:00Z".to_string(),
            tz: "America/Santiago".to_string(),
            fecha_pago: "2024-12-25T14:05:00Z".to_string(),
        },
        tipo_entrega: TipoEntrega { id: 1 }, // 1 = Delivery, 2 = Retiro, etc.
        drop_off: Some(DropOff {
            tipo_entrega: Some("Delivery".to_string()),
            direccion: Some("Calle Falsa 123".to_string()),
        }),
        courier: Courier { id_courier: -2 }, // -2 = Reparto Propio, ejemplo
        cliente: Cliente {
            telefono: Some("+56 9 1234 5678".to_string()),
            nombre: Some("Juan Pérez".to_string()),
            nro: Some("Depto. 202".to_string()),
        },
        sucursal: None, // No aplica si es delivery
        plataforma: Plataforma {
            codigo: Some("AGIL".to_string()),
            nombre: Some("AgilApp".to_string()),
        },
        correlativo: 1001,
        codigo: "ORD-999".to_string(),
        comercio: Comercio {
            nombre: Some("La Pizzería".to_string()),
        },
        pago: Pago {
            medio_pago: MedioPago {
                nombre: Some("Tarjeta".to_string()),
            },
        },
    };

    pdf(&orden_ejemplo);
    println!("espacio creado")
}