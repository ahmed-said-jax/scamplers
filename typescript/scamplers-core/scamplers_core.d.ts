/* tslint:disable */
/* eslint-disable */
export enum BlockPreservationMethod {
  FormaldehydeDerivativeFixation = 0,
}
export enum BlockType {
  Block = 0,
}
export enum ComplianceCommitteeType {
  Ibc = 0,
  Irb = 1,
  Iacuc = 2,
}
export enum EmbeddingMatrix {
  CarboxymethylCellulose = 0,
  OptimalCuttingTemperatureCompound = 1,
  Paraffin = 2,
}
export enum InstitutionOrdinalColumn {
  Name = 0,
}
export enum LabOrdinalColumn {
  Name = 0,
}
export enum PersonOrdinalColumn {
  Name = 0,
  Email = 1,
}
export enum Species {
  AmbystomaMexicanum = 0,
  CanisFamiliaris = 1,
  DrosophilaMelanogaster = 2,
  GasterosteusAculeatus = 3,
  HomoSapiens = 4,
  MusMusculus = 5,
  RattusNorvegicus = 6,
  SminthopsisCrassicaudata = 7,
}
export enum UserRole {
  AppAdmin = 0,
  ComputationalStaff = 1,
  BiologyStaff = 2,
}
export class Client {
  free(): void;
  send_new_institution(data: NewInstitution, api_key?: string | null): Promise<Institution>;
  send_new_person(data: NewPerson, api_key?: string | null): Promise<Person>;
  send_new_lab(data: NewLab, api_key?: string | null): Promise<Lab>;
  constructor(backend_url: string, token: string);
  send_new_ms_login(data: NewPerson): Promise<CreatedUser>;
}
export class CommitteeApproval {
  private constructor();
  free(): void;
  readonly institution: InstitutionSummary;
  readonly committee_type: ComplianceCommitteeType;
  readonly compliance_identifier: string;
}
export class CreatedUser {
  private constructor();
  free(): void;
  readonly id: string;
  readonly link: string;
  readonly name: string;
  readonly email: string;
  readonly orcid: string;
  readonly institution: Institution;
  readonly roles: any[];
  readonly api_key: string;
}
export class EmptyStringError {
  private constructor();
  free(): void;
}
export class Institution {
  private constructor();
  free(): void;
  readonly id: string;
  readonly link: string;
  readonly name: string;
}
export class InstitutionOrdering {
  private constructor();
  free(): void;
  static new(): InstitutionOrderingBuilder;
  column: InstitutionOrdinalColumn;
  descending: boolean;
}
/**
 * Builder for [`InstitutionOrdering`](struct.InstitutionOrdering.html).
 */
export class InstitutionOrderingBuilder {
  private constructor();
  free(): void;
  column(value: InstitutionOrdinalColumn): InstitutionOrderingBuilder;
  descending(value: boolean): InstitutionOrderingBuilder;
  /**
   * Builds a new `InstitutionOrdering`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): InstitutionOrdering;
}
export class InstitutionOrderingError {
  private constructor();
  free(): void;
  error(): string;
}
export class InstitutionQuery {
  free(): void;
  constructor();
  ids: string[];
  get name(): string;
  set name(value: string | null | undefined);
  order_by: InstitutionOrdering[];
  pagination: Pagination;
}
export class InstitutionReference {
  private constructor();
  free(): void;
  readonly id: string;
  readonly link: string;
}
export class InstitutionSummary {
  private constructor();
  free(): void;
  readonly id: string;
  readonly link: string;
  readonly name: string;
}
export class Lab {
  private constructor();
  free(): void;
  readonly id: string;
  readonly link: string;
  readonly name: string;
  readonly delivery_dir: string;
  readonly pi: PersonSummary;
  readonly members: PersonSummary[];
}
export class LabData {
  private constructor();
  free(): void;
  readonly id: string;
  readonly link: string;
  readonly name: string;
  readonly delivery_dir: string;
  readonly pi: PersonSummary;
}
export class LabOrdering {
  private constructor();
  free(): void;
  static new(): LabOrderingBuilder;
  column: LabOrdinalColumn;
  descending: boolean;
}
/**
 * Builder for [`LabOrdering`](struct.LabOrdering.html).
 */
export class LabOrderingBuilder {
  private constructor();
  free(): void;
  column(value: LabOrdinalColumn): LabOrderingBuilder;
  descending(value: boolean): LabOrderingBuilder;
  /**
   * Builds a new `LabOrdering`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): LabOrdering;
}
export class LabOrderingError {
  private constructor();
  free(): void;
  error(): string;
}
export class LabQuery {
  free(): void;
  constructor();
  ids: string[];
  get name(): string;
  set name(value: string | null | undefined);
  order_by: LabOrdering[];
  pagination: Pagination;
}
export class LabReference {
  private constructor();
  free(): void;
  readonly id: string;
  readonly link: string;
}
export class LabSummary {
  private constructor();
  free(): void;
  readonly id: string;
  readonly link: string;
  readonly name: string;
  readonly delivery_dir: string;
}
export class LabUpdate {
  private constructor();
  free(): void;
  static new(): LabUpdateBuilder;
  id: string;
  get name(): NonEmptyString | undefined;
  set name(value: NonEmptyString | null | undefined);
  get pi_id(): string;
  set pi_id(value: string | null | undefined);
  get delivery_dir(): NonEmptyString | undefined;
  set delivery_dir(value: NonEmptyString | null | undefined);
}
/**
 * Builder for [`LabUpdate`](struct.LabUpdate.html).
 */
export class LabUpdateBuilder {
  private constructor();
  free(): void;
  id(value: string): LabUpdateBuilder;
  name(value?: NonEmptyString | null): LabUpdateBuilder;
  pi_id(value?: string | null): LabUpdateBuilder;
  delivery_dir(value?: NonEmptyString | null): LabUpdateBuilder;
  /**
   * Builds a new `LabUpdate`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): LabUpdate;
}
export class LabUpdateError {
  private constructor();
  free(): void;
  error(): string;
}
export class LabUpdateWithMembers {
  private constructor();
  free(): void;
  static new(): LabUpdateWithMembersBuilder;
  update: LabUpdate;
  add_members: string[];
  remove_members: string[];
}
/**
 * Builder for [`LabUpdateWithMembers`](struct.LabUpdateWithMembers.html).
 */
export class LabUpdateWithMembersBuilder {
  private constructor();
  free(): void;
  update(value: LabUpdate): LabUpdateWithMembersBuilder;
  add_members(value: string[]): LabUpdateWithMembersBuilder;
  remove_members(value: string[]): LabUpdateWithMembersBuilder;
  /**
   * Builds a new `LabUpdateWithMembers`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): LabUpdateWithMembers;
}
export class LabUpdateWithMembersError {
  private constructor();
  free(): void;
  error(): string;
}
export class NewCommitteeApproval {
  private constructor();
  free(): void;
  static new(): NewCommitteeApprovalBuilder;
  get sample_id(): string;
  set sample_id(value: string | null | undefined);
  institution_id: string;
  committee_type: ComplianceCommitteeType;
  compliance_identifier: NonEmptyString;
}
/**
 * Builder for [`NewCommitteeApproval`](struct.NewCommitteeApproval.html).
 */
export class NewCommitteeApprovalBuilder {
  private constructor();
  free(): void;
  sample_id(value?: string | null): NewCommitteeApprovalBuilder;
  institution_id(value: string): NewCommitteeApprovalBuilder;
  committee_type(value: ComplianceCommitteeType): NewCommitteeApprovalBuilder;
  compliance_identifier(value: NonEmptyString): NewCommitteeApprovalBuilder;
  /**
   * Builds a new `NewCommitteeApproval`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): NewCommitteeApproval;
}
export class NewCommitteeApprovalError {
  private constructor();
  free(): void;
  error(): string;
}
export class NewInstitution {
  private constructor();
  free(): void;
  static new(): NewInstitutionBuilder;
  id: string;
  name: NonEmptyString;
}
/**
 * Builder for [`NewInstitution`](struct.NewInstitution.html).
 */
export class NewInstitutionBuilder {
  private constructor();
  free(): void;
  id(value: string): NewInstitutionBuilder;
  name(value: NonEmptyString): NewInstitutionBuilder;
  /**
   * Builds a new `NewInstitution`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): NewInstitution;
}
export class NewInstitutionError {
  private constructor();
  free(): void;
  error(): string;
}
export class NewLab {
  private constructor();
  free(): void;
  static new(): NewLabBuilder;
  name: NonEmptyString;
  pi_id: string;
  delivery_dir: NonEmptyString;
  member_ids: string[];
}
/**
 * Builder for [`NewLab`](struct.NewLab.html).
 */
export class NewLabBuilder {
  private constructor();
  free(): void;
  name(value: NonEmptyString): NewLabBuilder;
  pi_id(value: string): NewLabBuilder;
  delivery_dir(value: NonEmptyString): NewLabBuilder;
  member_ids(value: string[]): NewLabBuilder;
  /**
   * Builds a new `NewLab`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): NewLab;
}
export class NewLabError {
  private constructor();
  free(): void;
  error(): string;
}
export class NewPerson {
  private constructor();
  free(): void;
  static new(): NewPersonBuilder;
  name: NonEmptyString;
  email: string;
  get orcid(): NonEmptyString | undefined;
  set orcid(value: NonEmptyString | null | undefined);
  institution_id: string;
  get ms_user_id(): string;
  set ms_user_id(value: string | null | undefined);
  roles: any[];
}
/**
 * Builder for [`NewPerson`](struct.NewPerson.html).
 */
export class NewPersonBuilder {
  private constructor();
  free(): void;
  name(value: NonEmptyString): NewPersonBuilder;
  email(value: string): NewPersonBuilder;
  orcid(value?: NonEmptyString | null): NewPersonBuilder;
  institution_id(value: string): NewPersonBuilder;
  ms_user_id(value?: string | null): NewPersonBuilder;
  roles(value: any[]): NewPersonBuilder;
  /**
   * Builds a new `NewPerson`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): NewPerson;
}
export class NewPersonError {
  private constructor();
  free(): void;
  error(): string;
}
export class NonEmptyString {
  free(): void;
  constructor(s: string);
}
export class Pagination {
  free(): void;
  constructor(limit: bigint, offset: bigint);
  limit: bigint;
  offset: bigint;
}
export class Person {
  private constructor();
  free(): void;
  readonly id: string;
  readonly link: string;
  readonly name: string;
  readonly email: string;
  readonly orcid: string;
  readonly institution: Institution;
  readonly roles: any[];
}
export class PersonData {
  private constructor();
  free(): void;
  readonly id: string;
  readonly link: string;
  readonly name: string;
  readonly email: string;
  readonly orcid: string;
  readonly institution: Institution;
}
export class PersonDataUpdate {
  private constructor();
  free(): void;
  static new(): PersonDataUpdateBuilder;
  id: string;
  get name(): NonEmptyString | undefined;
  set name(value: NonEmptyString | null | undefined);
  get email(): string;
  set email(value: string | null | undefined);
  get ms_user_id(): string;
  set ms_user_id(value: string | null | undefined);
  get orcid(): NonEmptyString | undefined;
  set orcid(value: NonEmptyString | null | undefined);
  get institution_id(): string;
  set institution_id(value: string | null | undefined);
}
/**
 * Builder for [`PersonDataUpdate`](struct.PersonDataUpdate.html).
 */
export class PersonDataUpdateBuilder {
  private constructor();
  free(): void;
  id(value: string): PersonDataUpdateBuilder;
  name(value?: NonEmptyString | null): PersonDataUpdateBuilder;
  email(value?: string | null): PersonDataUpdateBuilder;
  ms_user_id(value?: string | null): PersonDataUpdateBuilder;
  orcid(value?: NonEmptyString | null): PersonDataUpdateBuilder;
  institution_id(value?: string | null): PersonDataUpdateBuilder;
  /**
   * Builds a new `PersonDataUpdate`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): PersonDataUpdate;
}
export class PersonDataUpdateError {
  private constructor();
  free(): void;
  error(): string;
}
export class PersonOrdering {
  private constructor();
  free(): void;
  static new(): PersonOrderingBuilder;
  column: PersonOrdinalColumn;
  descending: boolean;
}
/**
 * Builder for [`PersonOrdering`](struct.PersonOrdering.html).
 */
export class PersonOrderingBuilder {
  private constructor();
  free(): void;
  column(value: PersonOrdinalColumn): PersonOrderingBuilder;
  descending(value: boolean): PersonOrderingBuilder;
  /**
   * Builds a new `PersonOrdering`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): PersonOrdering;
}
export class PersonOrderingError {
  private constructor();
  free(): void;
  error(): string;
}
export class PersonQuery {
  free(): void;
  constructor();
  ids: string[];
  get name(): string;
  set name(value: string | null | undefined);
  get email(): string;
  set email(value: string | null | undefined);
  order_by: PersonOrdering[];
  pagination: Pagination;
}
export class PersonReference {
  private constructor();
  free(): void;
  readonly id: string;
  readonly link: string;
}
export class PersonSummary {
  private constructor();
  free(): void;
  readonly id: string;
  readonly link: string;
  readonly name: string;
  readonly email: string;
  readonly orcid: string;
}
export class PersonUpdate {
  private constructor();
  free(): void;
  static new(): PersonUpdateBuilder;
  data_update: PersonDataUpdate;
  add_roles: any[];
  remove_roles: any[];
}
/**
 * Builder for [`PersonUpdate`](struct.PersonUpdate.html).
 */
export class PersonUpdateBuilder {
  private constructor();
  free(): void;
  data_update(value: PersonDataUpdate): PersonUpdateBuilder;
  add_roles(value: any[]): PersonUpdateBuilder;
  remove_roles(value: any[]): PersonUpdateBuilder;
  /**
   * Builds a new `PersonUpdate`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): PersonUpdate;
}
export class PersonUpdateError {
  private constructor();
  free(): void;
  error(): string;
}
