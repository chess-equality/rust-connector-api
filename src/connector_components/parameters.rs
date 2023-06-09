use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct P<'a> {
    pub k: &'a str,
    pub v: Option<&'a str>,
}

pub type PSet<'a> = Vec<P<'a>>;

#[derive(Clone, Debug, PartialEq)]
pub struct Parameters<'a> {
    pub p_values: PSet<'a>,
}

impl<'a> Display for P<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.v {
            None => write!(f, "{}", self.k),
            Some(_) => write!(
                f,
                "{}",
                self.k.to_owned() + ":" + &*self.v.unwrap().to_owned()
            ),
        }
    }
}

impl<'a> Display for Parameters<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.p_values
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

#[cfg(test)]
mod tests {

    use crate::parameters::{PSet, Parameters, P};
    use std::iter::FromIterator;

    #[tokio::test]
    async fn with_some_values() {
        println!("\n##### with_some_values:");

        let params: Parameters = Parameters {
            p_values: PSet::from_iter([
                P {
                    k: "t_2m",
                    v: Some("C"),
                },
                P {
                    k: "precip_1h",
                    v: Some("mm"),
                },
            ]),
        };

        println!(">>>>>>>>>> params: {}", params);

        assert_eq!(params.to_string(), "t_2m:C,precip_1h:mm");

        assert_ne!(
            params.p_values,
            PSet::from_iter([P {
                k: "t_2m",
                v: Some("C")
            }])
        );
    }

    #[tokio::test]
    async fn with_none_values() {
        println!("\n##### with_none_values:");

        let mut p_values: PSet<'_> = PSet::new();
        let p1 = P {
            k: "precip_1h",
            v: Some("mm"),
        };
        let p2 = P {
            k: "wind_speed_10m",
            v: None,
        };
        p_values.push(p1);
        p_values.push(p2);

        let params: Parameters = Parameters { p_values };

        println!(">>>>>>>>>> params: {}", params);

        assert_eq!(params.to_string(), "precip_1h:mm,wind_speed_10m");

        assert_eq!(
            params.p_values,
            PSet::from_iter([
                P {
                    k: "precip_1h",
                    v: Some("mm")
                },
                P {
                    k: "wind_speed_10m",
                    v: None
                }
            ])
        );
    }
}
